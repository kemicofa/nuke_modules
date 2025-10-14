export default class Timer {
  #startMilliseconds: number;
  #endMilliseconds?: number;

  static start() {
    return new Timer();
  }

  /**
   * Ends the timer and calculates the elapsed time once.
   * Further calls will always return the same value.
   *
   * @returns the elapsed time in seconds
   */
  end() {
    if (!this.#endMilliseconds) {
      this.#endMilliseconds = new Date().getTime();
    }

    return (this.#endMilliseconds - this.#startMilliseconds) / 1000;
  }

  private constructor() {
    this.#startMilliseconds = new Date().getTime();
  }
}
