import "codemirror/lib/codemirror.css";

import codemirror from "codemirror";
import {h, Component} from "preact";

import TlaMode from "../TlaMode";


export default class Codemirror extends Component {
  constructor(props) {
    super(props);
  }


  componentDidMount() {
    // register TLA mode
    const tlaMode = new TlaMode();
    codemirror.defineMode('tla', () => {
      return {
        token: (stream, state) => {
          const line = stream.lineOracle.line + 1;
          const column = stream.column() + 1;
          stream.skipToEnd();
          return tlaMode.getStyle(line, column);
        }
      };
    });

    this.codeMirror = codemirror.fromTextArea(
      this.textAreaNode,
      this.props.options
    );
    this.codeMirror.getDoc().setValue(this.props.value);
    this.codeMirror.on("change", (doc, change) => {
      console.log("change", change);
      const code = doc.getValue();
      tlaMode.updateParseTree(code);
    });
  }


  componentWillReceiveProps(props) {
    if (this.codeMirror && this.props.value !== props.value) {
      this.codeMirror.getDoc().setValue(props.value);
    }
  }


  componentWillUnmount() {
    if (this.codeMirror) this.codeMirror.toTextArea();
  }


  render() {
    return (
      <textarea
        ref={ref => this.textAreaNode = ref}
        autoComplete="off"
        autoFocus={this.props.autoFocus}
        mode="tla"
      />);
  }
}
