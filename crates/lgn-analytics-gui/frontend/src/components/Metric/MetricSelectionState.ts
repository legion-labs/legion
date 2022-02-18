export class MetricSelectionState {
  selected: boolean;
  name: string;
  unit: string;
  constructor(name: string, unit: string) {
    this.name = name;
    this.unit = unit;
    this.selected = true;
  }
}
