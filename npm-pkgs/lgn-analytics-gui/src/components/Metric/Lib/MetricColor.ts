const colors = [
  "#79CDAF",
  "#3984B6",
  "#F0A04B",
  "#EC7C92",
  "#7E84FA",
  "#62CB5A",
  "#147AF3",
  "#A73B8F",
  "#FBDB68",
  "#E5632F",
  "#008F5D",
];

export function getMetricColor(index: number) {
  return colors[index % colors.length];
}
