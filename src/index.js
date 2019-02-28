import './style';
import { Component } from 'preact';
import Editor from './Editor';

export default class App extends Component {
  render() {
    const text = "---- test ----\n====";
    const options = {
      lineNumbers: true,
    };
    return (
      <div>
        <h1>TLA+ UI</h1>
        <Editor name="TLA+"
          defaultValue={text}
          autoFocus={true}
          options={options} />
      </div>
    );
  }
}
