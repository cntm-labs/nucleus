import { Nucleus } from '../nucleus'
export class NucleusUserButton extends HTMLElement {
  connectedCallback() {
    const user = Nucleus.getUser()
    this.innerHTML = user
      ? \`<button class="nucleus-user-btn">\${user.first_name?.[0] ?? user.email[0]}</button>\`
      : ''
  }
}
if (typeof customElements !== 'undefined') customElements.define('nucleus-user-button', NucleusUserButton)
