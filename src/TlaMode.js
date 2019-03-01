import * as CodeMirror from 'codemirror';
import * as Tla from 'tla-parser';

export function registerMode() {
  CodeMirror.defineMode('tla', function() {
    return {
      token: function(stream) {
        const token = stream.peek();
        stream.skipToEnd();
        return Tla.parse(token);
      }
    };
  })
}
