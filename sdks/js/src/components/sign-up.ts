export class NucleusSignUp extends HTMLElement {
  connectedCallback() {
    this.innerHTML = \`
      <form id="nucleus-sign-up-form">
        <input type="text" placeholder="Full Name" required />
        <input type="email" placeholder="Email" required />
        <input type="password" placeholder="Password" minlength="8" required />
        <button type="submit">Create Account</button>
      </form>\`
  }
}
if (typeof customElements !== 'undefined') customElements.define('nucleus-sign-up', NucleusSignUp)
