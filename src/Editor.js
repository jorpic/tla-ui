import 'codemirror/lib/codemirror.css';
import { Component } from 'preact';
import * as CodeMirror from 'codemirror';
import * as Tla from './TlaMode';


export default class Editor  extends Component {
  componentDidMount() {
    Tla.registerMode();
    this.codeMirror = CodeMirror.fromTextArea(
      this.textareaNode,
      this.props.options);
  }


  componentWillUnmount() {
    if (this.codeMirror) this.codeMirror.toTextarea();
  }


  render() {
    return (
      <textarea
        ref={ref => this.textareaNode = ref}
        name={this.props.name}
        defaultValue={this.props.value}
        autoComplete="off"
        autoFocus={this.props.autoFocus}
        mode="tla"
      />)
  }
}
