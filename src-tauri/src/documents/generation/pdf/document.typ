#set page(
  paper: "a4",
  margin: (x: 22mm, y: 22mm),
  footer: context align(right, text(size: 8.5pt, fill: luma(140), counter(page).display("1 / 1", both: true))),
)
#set par(justify: false, leading: 0.78em, spacing: 1em)
#set heading(numbering: none)
#show heading: set block(above: 1.35em, below: 0.6em, sticky: true)
#show heading.where(level: 1): set text(size: 18pt, weight: "bold")
#show heading.where(level: 2): set text(size: 14pt, weight: "bold")
#show heading.where(level: 3): set text(size: 12pt, weight: "bold")
#show raw.where(block: true): set block(
  width: 100%,
  fill: luma(248),
  inset: 9pt,
  radius: 4pt,
  stroke: 0.5pt + luma(226),
)
#show raw.where(block: true): set text(size: 9pt)
#show raw.where(block: false): box.with(
  fill: luma(244),
  inset: (x: 3pt, y: 0pt),
  outset: (y: 2pt),
  radius: 2pt,
)
#set table(
  inset: (x: 7pt, y: 6pt),
  stroke: 0.5pt + luma(210),
  fill: (_, y) => if y == 0 { luma(246) },
)
#set line(stroke: 0.5pt + luma(220))
