export class DeclarationIterator {
  str: string;
  idx: number;

  constructor(str: string) {
    this.str = str;
    this.idx = 0;
  }

  hasNext() {
    return (
      this.idx <= this.str.length &&
      this.str.substring(this.idx).search('export ') !== -1
    );
  }

  async next() {
    if (!this.hasNext()) {
      throw Error('no next element.');
    }

    const start = this.idx + this.str.substring(this.idx).search('export ');
    const isInterface = this.str
      .substring(start)
      .startsWith('export interface');

    const firstParenPos = start + this.str.substring(start).search('{');
    const firstAssignPos = start + this.str.substring(start).search('=');
    const firstSemiColPos = start + this.str.substring(start).search(';');
    const isComplexType =
      firstSemiColPos > firstParenPos && firstAssignPos < firstParenPos;
    const isComplex = isInterface || isComplexType;

    const namePos = isInterface
      ? { from: start + 'export interface'.length, to: firstParenPos }
      : { from: start + 'export type'.length, to: firstAssignPos };
    const name = this.str.substring(namePos.from, namePos.to).trim();

    if (isComplex) {
      // match parenthesis
      this.idx = start;
      let open = 0;
      do {
        while (!['{', '}'].includes(this.str[this.idx])) {
          this.idx = this.idx + 1;
        }

        if (this.str[this.idx] === '{') {
          open++;
        }

        if (this.str[this.idx] === '}') {
          open--;
        }

        this.idx++;
      } while (open > 0 || (!isInterface && this.str[this.idx] !== ';'));

      return { content: this.str.substring(start, this.idx), name };
    } else {
      // simple declaration
      this.idx = firstSemiColPos + 1;
      return { content: this.str.substring(start, this.idx - 1), name };
    }
  }
}