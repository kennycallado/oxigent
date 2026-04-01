import { LitElement, html, css } from "lit";

export class RustKanbanApp extends LitElement {
  static styles = css`
    :host {
      display: block;
      height: 100vh;
    }
  `;

  render() {
    return html`<h1>Rust Kanban Desktop</h1>`;
  }
}

customElements.define("rust-kanban-app", RustKanbanApp);