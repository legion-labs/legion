export function bytesOut(x) {
  return btoa(JSON.stringify(x));
}

export function bytesIn(x) {
  return JSON.parse(atob(x));
}
