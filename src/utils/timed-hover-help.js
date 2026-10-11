// One passive help bubble for native title controls and editor descriptions.
// Keep the target after dismissal so stationary hover cannot reopen it.
export function installTimedHoverHelp(doc = document) {
  const win = doc.defaultView
  let target = null
  let bubble = null
  let showTimer = null
  let hideTimer = null
  const content = element =>
    element?.getAttribute('data-editor-help') ?? element?.getAttribute('data-hover-help')
  const dismiss = () => {
    win.clearTimeout(showTimer)
    win.clearTimeout(hideTimer)
    bubble?.remove()
    bubble = null
  }
  const convertTitle = element => {
    if (!element.hasAttribute('title')) return
    const text = element.getAttribute('data-editor-help') ?? element.getAttribute('title')
    if (text) {
      element.setAttribute('data-hover-help', text)
      element.setAttribute('aria-description', text)
      // Native title was also the accessible name for some icon-only controls.
      if (
        element.matches('button, input, select, a') &&
        (!element.hasAttribute('aria-label') ||
          element.getAttribute('aria-label') === element.getAttribute('data-hover-help-name')) &&
        !element.hasAttribute('aria-labelledby') &&
        !element.textContent.trim()
      ) {
        element.setAttribute('aria-label', text)
        element.setAttribute('data-hover-help-name', text)
      }
    } else {
      element.removeAttribute('data-hover-help')
      element.removeAttribute('aria-description')
      if (element.getAttribute('aria-label') === element.getAttribute('data-hover-help-name'))
        element.removeAttribute('aria-label')
      element.removeAttribute('data-hover-help-name')
    }
    element.removeAttribute('title')
  }
  const scan = node => {
    if (node.nodeType !== 1) return
    convertTitle(node)
    node.querySelectorAll('[title]').forEach(convertTitle)
  }
  scan(doc.documentElement)
  const observer = new win.MutationObserver(records => {
    for (const record of records) {
      if (record.type === 'attributes') convertTitle(record.target)
      else record.addedNodes.forEach(scan)
    }
    if (target && !target.isConnected) {
      dismiss()
      target = null
    } else if (target && !content(target)) dismiss()
    else if (bubble && bubble.textContent !== content(target)) dismiss()
  })
  observer.observe(doc.documentElement, {
    subtree: true,
    childList: true,
    attributes: true,
    attributeFilter: ['title', 'data-editor-help', 'data-hover-help'],
  })
  const enter = event => {
    if (event.pointerType === 'touch') return
    const next = event.target.closest?.('[data-editor-help], [data-hover-help]')
    if (!next || next === target || !content(next)) return
    dismiss()
    target = next
    showTimer = win.setTimeout(() => {
      if (!target?.isConnected || !content(target)) return
      bubble = doc.createElement('div')
      bubble.className = 'lrcget-hover-help'
      bubble.setAttribute('role', 'tooltip')
      bubble.textContent = content(target)
      doc.body.append(bubble)
      const anchor = target.getBoundingClientRect()
      const box = bubble.getBoundingClientRect()
      bubble.style.left = `${Math.max(8, Math.min(anchor.left, win.innerWidth - box.width - 8))}px`
      const below = anchor.bottom + 8
      bubble.style.top = `${Math.max(8, below + box.height <= win.innerHeight - 8 ? below : anchor.top - box.height - 8)}px`
      hideTimer = win.setTimeout(dismiss, 2000)
    }, 350)
  }
  const leave = event => {
    if (!target || target.contains(event.relatedTarget)) return
    if (target.contains(event.target)) {
      dismiss()
      target = null
    }
  }
  const key = event => {
    if (event.key === 'Escape') dismiss()
  }
  const listeners = [
    [doc, 'pointerover', enter],
    [doc, 'pointerout', leave],
    [doc, 'pointerdown', dismiss],
    [doc, 'scroll', dismiss],
    [doc, 'keydown', key],
    [doc, 'visibilitychange', dismiss],
    [win, 'blur', dismiss],
    [win, 'resize', dismiss],
  ]
  listeners.forEach(([owner, type, listener]) => owner.addEventListener(type, listener, true))
  return () => {
    dismiss()
    observer.disconnect()
    listeners.forEach(([owner, type, listener]) => owner.removeEventListener(type, listener, true))
    target = null
  }
}
