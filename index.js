import './style';
import 'codemirror/lib/codemirror.css';
import * as codemirror from 'codemirror';


import { Component } from 'preact';

class Codemirror extends Component {
  componentDidMount() {
    this.codeMirror = codemirror.fromTextArea(this.textareaNode, this.props.options);
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
      />)
  }
}

export default class App extends Component {
  render() {
    const text = "---- test ----\n====";
    const options = {
      lineNumbers: true,
    };
    return (
      <div>
        <h1>TLA+ UI</h1>
        <Codemirror name="TLA+"
          defaultValue={text}
          autoFocus={true}
          options={options} />
      </div>
    );
  }
}
