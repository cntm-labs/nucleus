export class NucleusSignIn extends HTMLElement {
  connectedCallback() {
    this.innerHTML = \`
      <form id="nucleus-sign-in-form">
        <input type="email" placeholder="Email" required />
        <input type="password" placeholder="Password" required />
        <button type="submit">Sign In</button>
      </form>\`
    this.querySelector('form')?.addEventListener('submit', (e) => {
      e.preventDefault()
      this.dispatchEvent(new CustomEvent('nucleus:signin', { bubbles: true }))
    })
  }
}
if (typeof customElements !== 'undefined') customElements.define('nucleus-sign-in', NucleusSignIn)
