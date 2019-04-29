import * as CodeMirror from 'codemirror';
import * as Tla from 'tla-parser';

export class Mode {

  constructor() {
    this.parseTree = null;
    this.register()
  }

  register() {
    CodeMirror.defineMode('tla', () => {
      return {
        token: (stream, state) => {
          const line = stream.lineOracle.line + 1;
          const column = stream.column() + 1;
          stream.skipToEnd();
          return this.getStyle(line, column);
        }
      };
    })
  }

  updateParseTree(code) {
    this.parseTree = Tla.parse(code);
    console.log("Parse tree", this.parseTree);
  }

  getStyle(line, col) {
    const style = this.parseTree && this.parseTree.get_style(line, col);
    console.log("Style", style);
    return style;
  }
}
