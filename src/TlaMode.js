import * as CodeMirror from 'codemirror';

export function registerMode() {
  CodeMirror.defineMode('tla', function() {
    return {
      token: function(stream) {
        const TOKEN_NAMES = {'#': 'comment'};
        const token = TOKEN_NAMES[stream.peek()];
        stream.skipToEnd();
        return token;
      }
    };
  })
}
