const MASK_CHARACTER = {
  line: maskLineCharacter,
  block: maskBlockCharacter,
  string: maskStringCharacter,
  code: maskCodeCharacter,
};

export function maskRust(source, maskStrings) {
  const chars = source.split("");
  const state = { mode: "code", blockDepth: 0, escaped: false };
  for (let index = 0; index < source.length; index += 1) {
    const char = source[index];
    const next = source[index + 1] ?? "";
    index += MASK_CHARACTER[state.mode]({ state, chars, index, char, next, maskStrings });
  }
  return chars.join("");
}

function maskLineCharacter({ state, chars, index, char }) {
  if (char === "\n") state.mode = "code";
  else chars[index] = " ";
  return 0;
}

function maskBlockCharacter({ state, chars, index, char, next }) {
  if (char === "/" && next === "*") {
    state.blockDepth += 1;
    maskPair(chars, index);
    return 1;
  }
  if (char === "*" && next === "/") {
    state.blockDepth -= 1;
    maskPair(chars, index);
    if (state.blockDepth === 0) state.mode = "code";
    return 1;
  }
  if (char !== "\n") chars[index] = " ";
  return 0;
}

function maskStringCharacter({ state, chars, index, char, maskStrings }) {
  if (state.escaped) state.escaped = false;
  else if (char === "\\") state.escaped = true;
  else if (char === '"') state.mode = "code";
  if (maskStrings && char !== "\n") chars[index] = " ";
  return 0;
}

function maskCodeCharacter({ state, chars, index, char, next, maskStrings }) {
  if (char === "/" && next === "/") {
    state.mode = "line";
    maskPair(chars, index);
    return 1;
  }
  if (char === "/" && next === "*") {
    state.mode = "block";
    state.blockDepth = 1;
    maskPair(chars, index);
    return 1;
  }
  if (char === '"') {
    state.mode = "string";
    if (maskStrings) chars[index] = " ";
  }
  return 0;
}

function maskPair(chars, index) {
  chars[index] = " ";
  chars[index + 1] = " ";
}
