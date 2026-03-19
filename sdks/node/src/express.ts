import type { Request, Response, NextFunction } from 'express'
import { verifyToken } from './verify'
import { NucleusConfig, NucleusClaims } from './types'

declare global {
  namespace Express {
    interface Request {
      nucleusClaims?: NucleusClaims
    }
  }
}

export function nucleusMiddleware(config: NucleusConfig) {
  return async (req: Request, _res: Response, next: NextFunction) => {
    const authHeader = req.headers.authorization
    if (!authHeader?.startsWith('Bearer ')) {
      return next()
    }
    try {
      const token = authHeader.slice(7)
      req.nucleusClaims = await verifyToken(token, config)
    } catch {
      // Token invalid — continue without claims
    }
    next()
  }
}

export function requireAuth(config: NucleusConfig) {
  return async (req: Request, res: Response, next: NextFunction) => {
    const authHeader = req.headers.authorization
    if (!authHeader?.startsWith('Bearer ')) {
      res.status(401).json({ error: { code: 'auth/token_invalid', message: 'Missing authorization header' } })
      return
    }
    try {
      const token = authHeader.slice(7)
      req.nucleusClaims = await verifyToken(token, config)
      next()
    } catch {
      res.status(401).json({ error: { code: 'auth/token_invalid', message: 'Invalid or expired token' } })
    }
  }
}

export type { NucleusConfig, NucleusClaims }
