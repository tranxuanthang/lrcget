let initialized = false;
let _kuroshiro: Kuroshiro | null = null

const kuroshiro = (async () => {
  if (_kuroshiro && initialized) {
    return _kuroshiro;
  }


  _kuroshiro = new Kuroshiro();
  await _kuroshiro.init(
    new KuromojiAnalyzer({
      dictPath: 'https://cdn.jsdelivr.net/npm/kuromoji@0.1.2/dict/',
    }),
  );
  initialized = true;
  return _kuroshiro;
});

export const romanizeJapanese = async (line: string) =>
  (await kuroshiro()).convert(line, {
    to: 'romaji',
    mode: 'spaced',
  });

export const canonicalize = (text: string) => {
  return (
    text
      // `hi  there` => `hi there`
      .replaceAll(/\s+/g, ' ')

      // `( a )` => `(a)`
      .replaceAll(/([([]) ([^ ])/g, (_, symbol, a) => `${symbol}${a}`)
      .replaceAll(/([^ ]) ([)\]])/g, (_, a, symbol) => `${a}${symbol}`)

      // `can ' t` => `can't`
      .replaceAll(
        /([Ii]) (') ([^ ])|(n) (') (t)(?= |$)|(t) (') (s)|([^ ]) (') (re)|([^ ]) (') (ve)|([^ ]) (-) ([^ ])/g,
        (m, ...groups) => {
          for (let i = 0; i < groups.length; i += 3) {
            if (groups[i]) {
              return groups.slice(i, i + 3).join('');
            }
          }

          return m;
        },
      )
      // `Stayin ' still` => `Stayin' still`
      .replaceAll(/in ' ([^ ])/g, (_, char) => `in' ${char}`)
      .replaceAll("in ',", "in',")

      .replaceAll(", ' cause", ", 'cause")

      // `hi , there` => `hi, there`
      .replaceAll(/([^ ]) ([.,!?])/g, (_, a, symbol) => `${a}${symbol}`)

      // `hi " there "` => `hi "there"`
      .replaceAll(
        /"([^"]+)"/g,
        (_, content) =>
          `"${typeof content === 'string' ? content.trim() : content}"`,
      )
      .trim()
  );
};
