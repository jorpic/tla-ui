import "codemirror/lib/codemirror.css";

import codemirror from "codemirror";
import {h, Component} from "preact";


export default class Codemirror extends Component {
  constructor(props) {
    super(props);
  }


  componentDidMount() {
    this.codeMirror = codemirror.fromTextArea(
      this.textAreaNode,
      this.props.options
    );
    this.codeMirror.getDoc().setValue(this.props.value);
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
      />);
  }
}
