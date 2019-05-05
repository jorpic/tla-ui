import { parse, default as initParser } from "tla-parser";
import wasm from  "file-loader!tla-parser/tla_parser_bg.wasm";


export default class Mode {
  constructor() {
    this.parse = () => null;
    this.parseTree = null;

    initParser(wasm) // fetch and init parser
      .then(res => {
        this.parse = parse;
        console.log("tla-parser loaded:", res);
      })
      .catch(err => console.error("initParser:", err));
  }

  updateParseTree(code) {
    this.parseTree = parse(code);
    console.log("Parse tree", this.parseTree);
  }

  getStyle(line, col) {
    const style = this.parseTree && this.parseTree.get_style(line, col);
    console.log("Style", style);
    return style;
  }
}
