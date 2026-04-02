import { LitElement, html, css } from "lit";

export class OxigentApp extends LitElement {
  static styles = css`
    :host {
      display: block;
      height: 100vh;
    }
  `;

  render() {
    return html`<h1>Oxigent</h1>`;
  }
}

customElements.define("oxigent-app", OxigentApp);
