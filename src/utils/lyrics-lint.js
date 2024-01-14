import { parseLine } from 'lrc-kit'

export const executeLint = (source) => {
  const lines = source.split('\n').map(line => line.trim())
  const problems = []
  let lastNonEmptyLine = null

  lines.forEach((content, index) => {
    if (content) {
      const parsed = parseLine(content)
      if (parsed.type === 'INVALID') {
        problems.push({
          line: index + 1,
          severity: 'error',
          message: 'Line is not synchronized or invalid tag syntax'
        })
      }

      if (parsed.type === 'TIME' && ((parsed.content.endsWith('.') && !parsed.content.endsWith('...')) || parsed.content.endsWith(','))) {
        problems.push({
          line: index + 1,
          severity: 'error',
          message: 'Line should not end with a punctuation such as comma and dot'
        })
      }

      if (parsed.type === 'TIME' && parsed.content && !content.match(/^\[(.*)\]\s/)) {
        problems.push({
          line: index + 1,
          severity: 'error',
          message: 'Expect a space between the time tag and the content'
        })
      }

      lastNonEmptyLine = { content, lineIndex: index }
    } else {
      if (index < lines.length - 1) {
        problems.push({
          line: index + 1,
          severity: 'error',
          message: 'Unnecessary empty line'
        })
      }
    }
  })


  if (lines.length > 1 && lastNonEmptyLine) {
    const parsed = parseLine(lastNonEmptyLine.content)

    if (parsed.type !== 'TIME' || !!parsed.content.trim()) {
      problems.push({
        line: lastNonEmptyLine.lineIndex + 1,
        severity: 'error',
        message: 'Expect a synchronized empty line to mark the end of lyrics'
      })
    }
  }

  return problems
}
