export class TimelineMinimapViewport {
  x = 0;
  y = 0;
  height = 0;
  width = 0;

  set(x: number, y: number, width: number, height: number) {
    this.x = x;
    this.y = y;
    this.width = width;
    this.height = height;
  }
}
