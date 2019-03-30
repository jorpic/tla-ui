import './style.css';
import 'codemirror/lib/codemirror.css';
import * as CodeMirror from 'codemirror';
// import * as Tla from './TlaMode';

const text = "---- MODULE xxx ----\nEXTENDS Integers, TLC, Sequences\n====";

const options = {
  lineNumbers: true,
  mode: "tla",
  autofocus: true,
};

const tlaMode = new Tla.Mode();
const textarea = document.getElementById("codemirror");
const codeMirror = CodeMirror.fromTextArea(textarea, options);
codeMirror.on("change", function(doc, change) {
  const code = doc.getValue();
  tlaMode.updateParseTree(code);
});
codeMirror.setValue(text);
