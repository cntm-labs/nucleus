import { useState } from 'react'
import type { NucleusOrganization } from '../client/types'

export function useOrganizationList() {
  const [organizations] = useState<NucleusOrganization[]>([])
  const [isLoaded] = useState(false)

  // TODO: fetch organizations from API

  return {
    organizations,
    isLoaded,
  }
}
