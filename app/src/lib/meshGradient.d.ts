export class Gradient {
  el: HTMLCanvasElement;
  conf: { playing: boolean };
  play(): void;
  pause(): void;
  initGradient(selector: string): this;
  toggleColor(index: number): void;
  updateFrequency(freq: number): void;
}
