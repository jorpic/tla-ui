import { h, Component } from "preact";

export default class App extends Component {
  constructor(props) {
    super(props);
    this.state = {
      file: null
    };
  }


  openFile = ev => {
    const files = ev.target.files;
    if(files.length > 0) {
      const reader = new FileReader();
      reader.readAsText(files[0], "UTF-8");
      reader.onload = evt => this.setState({
        file: {
          name: files[0].name,
          code: evt.target.result
      }});
      reader.onerror = evt => this.setState({file: {error: true}});
    }
  }

  render = () => (
    <div>
      <nav class="navbar has-shadow" role="navigation" aria-label="main navigation">
        <div class="navbar-brand">
          <a class="navbar-item" href="#">
            TLA+
          </a>
        </div>

        <div class="navbar-menu is-active">
          <div class="navbar-start">
            <div class="navbar-item has-dropdown is-hoverable">
              <a class="navbar-link">
                File
              </a>
              <div class="navbar-dropdown">
                <a class="navbar-item file">
                  <input class="file-input" type="file" accept=".tla"
                    onChange={this.openFile}
                  />
                  Open local…
                </a>
              </div>
            </div>
            <a class="navbar-item">
              Config
            </a>
          </div>

          <div class="navbar-end"></div>
        </div>
      </nav>
      <div class="container">
        <div class="content">
          {this.state.file && this.state.file.name}
          {this.state.file && this.state.file.code}
        </div>
      </div>
    </div>
  )
}
