export function tokenPattern(shape) {
  return shape ? new RegExp(`(["'\`])(${shape})\\1`, "gu") : null;
}

export function matched(source, pattern) {
  return pattern ? new Set([...source.matchAll(pattern)].map((match) => match[2])) : new Set();
}
