import { NucleusConfig, NucleusUser } from './types'
export class Nucleus {
  private static config: NucleusConfig
  private static user: NucleusUser | null = null
  private static token: string | null = null
  static configure(config: NucleusConfig) { Nucleus.config = config }
  static getUser(): NucleusUser | null { return Nucleus.user }
  static async getToken(): Promise<string | null> { return Nucleus.token }
  static isSignedIn(): boolean { return Nucleus.user !== null }
  static async signOut(): Promise<void> { Nucleus.user = null; Nucleus.token = null }
}
