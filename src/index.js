import './style.css';
import 'codemirror/lib/codemirror.css';
import * as CodeMirror from 'codemirror';
import * as Tla from './TlaMode';

Tla.registerMode();
const text = "---- test ----\n====";
const options = {
  lineNumbers: true,
  mode: "tla",
  autoFocus: true,
  value: text
};

const textarea = document.getElementById("codemirror");
const codeMirror = CodeMirror.fromTextArea(textarea, options);
