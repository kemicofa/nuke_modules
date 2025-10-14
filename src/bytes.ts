const units = ["B", "K", "M", "G", "T"];

export function humanSize(bytes: number) {
  let i = 0;
  while (bytes >= 1024 && i < units.length - 1) {
    bytes /= 1024;
    i++;
  }

  // Set precision for anything under M
  const precision = i >= 2 ? 0 : 1;
  return `${bytes.toFixed(precision)}${units[i]}`;
}
