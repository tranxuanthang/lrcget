export const isSynchronizedLyrics = (lyrics) => {
  // Match either timestamp tags [00:00.00] or metadata tags [xx:value]
  return /^\[(?:\d{2}:\d{2}[.:]\d{2,3}|[a-z]+:.+?)\]/.test(lyrics);
};

export const detectStandard = (lyrics) => {
  // Define format patterns
  const formats = [
    {
      name: "[mm:ss.xx] text",
      space: true,
      msPrecision: 2,
      regex: /^\[\d{2}:\d{2}\.\d{2}\](?= [^ ])/,
    },
    {
      name: "[mm:ss.xx]text",
      space: false,
      msPrecision: 2,
      regex: /^\[\d{2}:\d{2}\.\d{2}\](?=[^ ])/,
    },
    {
      name: "[mm:ss.xxx] text",
      space: true,
      msPrecision: 3,
      regex: /^\[\d{2}:\d{2}\.\d{3}\](?= [^ ])/,
    },
    {
      name: "[mm:ss.xxx]text",
      space: false,
      msPrecision: 3,
      regex: /^\[\d{2}:\d{2}\.\d{3}\](?=[^ ])/,
    },
  ];

  // Split into lines and find first line with timestamp
  const lines = lyrics.split("\n");
  const firstTimestampLine = lines.find((line) =>
    /^\[\d{2}:\d{2}[.:]\d{2,3}\]/.test(line),
  );

  // If no timestamp line is found (e.g. the lyrics are not synchronized), return the first format
  if (!firstTimestampLine) {
    return formats[0];
  }

  // Find matching format
  return (
    formats.find((format) => format.regex.test(firstTimestampLine)) ||
    formats[0]
  );
};

export const timestampToString = (seconds, precision = 2) => {
  // Handle negative or invalid input
  if (!seconds || seconds < 0) seconds = 0;

  // Get minutes and remaining seconds
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = seconds % 60;

  // Format minutes and seconds with padStart to ensure 2 digits
  const mm = String(minutes).padStart(2, "0");

  // Format seconds with 2 digits before decimal and 2 after
  const wholeSecs = Math.floor(remainingSeconds);
  const decimal = (remainingSeconds % 1).toFixed(precision).substring(2);
  const ss = String(wholeSecs).padStart(2, "0");

  return `${mm}:${ss}.${decimal}`;
};
