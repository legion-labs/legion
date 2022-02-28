export class MetricSelectionState {
  selected: boolean;
  hidden: boolean;
  name: string;
  unit: string;
  constructor(name: string, unit: string, selected: boolean, hidden: boolean) {
    this.name = name;
    this.unit = unit;
    this.selected = selected;
    this.hidden = hidden;
  }
}
