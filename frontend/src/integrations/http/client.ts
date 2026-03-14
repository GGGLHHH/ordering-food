import ky, { HTTPError, type KyInstance, type Options, TimeoutError } from 'ky'

import type { ErrorEnvelope } from '#/contracts/generated'

import {
  ApiError,
  createHttpApiError,
  createNetworkApiError,
  createTimeoutApiError,
  createUnknownApiError,
  isAbortError,
} from './error'

const AUTH_RECOVERY_PATHS = new Set(['auth/login', 'auth/logout', 'auth/refresh'])
const DEFAULT_TIMEOUT_MS = 10_000

export type AuthMode = 'none' | 'optional' | 'required'

export interface ApiClientContext {
  baseUrl?: string
  fetch?: typeof fetch
  headers?: HeadersInit
  onAuthRedirect?: (href: string) => void
}

export interface ApiRequestOptions {
  authMode?: AuthMode
  headers?: HeadersInit
  json?: Options['json']
  method?: Options['method']
  searchParams?: Options['searchParams']
  signal?: AbortSignal
}

interface InternalApiRequestOptions extends ApiRequestOptions {
  hasRetried?: boolean
  skipAuthRefresh?: boolean
}

export interface ApiClient {
  requestJson<TResponse>(path: string, options?: ApiRequestOptions): Promise<TResponse>
  requestVoid(path: string, options?: ApiRequestOptions): Promise<void>
}

let browserApiClient: ApiClient | undefined

export function createApiClient(context: ApiClientContext = {}): ApiClient {
  const baseUrl = resolveBaseUrl(context.baseUrl)
  const client = ky.create({
    credentials: 'include',
    fetch: context.fetch,
    headers: context.headers,
    retry: 0,
    throwHttpErrors: true,
    timeout: DEFAULT_TIMEOUT_MS,
  })

  let refreshPromise: Promise<void> | null = null

  async function requestJson<TResponse>(
    path: string,
    options: ApiRequestOptions = {},
  ): Promise<TResponse> {
    const response = await execute(path, options, 'json')
    return (await response.json()) as TResponse
  }

  async function requestVoid(path: string, options: ApiRequestOptions = {}): Promise<void> {
    await execute(path, options, 'void')
  }

  async function execute(
    path: string,
    options: InternalApiRequestOptions,
    responseMode: 'json' | 'void',
  ): Promise<Response> {
    const normalizedPath = normalizePath(path)

    try {
      return await client(buildUrl(baseUrl, normalizedPath), buildKyOptions(options))
    } catch (error) {
      if (shouldRefreshSession(error, normalizedPath, options)) {
        try {
          refreshPromise ??= refreshSession(client, baseUrl)
          await refreshPromise
        } catch (refreshError) {
          const normalizedError = await normalizeError(refreshError)
          if (options.authMode === 'required') {
            redirectToLogin(context.onAuthRedirect)
          }
          throw normalizedError
        } finally {
          refreshPromise = null
        }

        return execute(
          normalizedPath,
          {
            ...options,
            hasRetried: true,
          },
          responseMode,
        )
      }

      const normalizedError = await normalizeError(error)
      if (
        options.authMode === 'required' &&
        normalizedError instanceof ApiError &&
        normalizedError.isUnauthorized
      ) {
        redirectToLogin(context.onAuthRedirect)
      }
      throw normalizedError
    }
  }

  return {
    requestJson,
    requestVoid,
  }
}

export function requestJson<TResponse>(
  path: string,
  options: ApiRequestOptions = {},
): Promise<TResponse> {
  return getBrowserApiClient().requestJson<TResponse>(path, options)
}

export function requestVoid(path: string, options: ApiRequestOptions = {}): Promise<void> {
  return getBrowserApiClient().requestVoid(path, options)
}

function getBrowserApiClient(): ApiClient {
  browserApiClient ??= createApiClient()
  return browserApiClient
}

async function normalizeError(error: unknown): Promise<ApiError | DOMException> {
  if (isAbortError(error)) {
    return error
  }

  if (error instanceof ApiError) {
    return error
  }

  if (error instanceof HTTPError) {
    return await toHttpApiError(error)
  }

  if (error instanceof TimeoutError) {
    return createTimeoutApiError(error)
  }

  if (error instanceof TypeError) {
    return createNetworkApiError(error)
  }

  return createUnknownApiError(error)
}

async function toHttpApiError(error: HTTPError): Promise<ApiError> {
  const envelope = await readErrorEnvelope(error.response)
  return createHttpApiError(error.response.status, envelope, error)
}

async function readErrorEnvelope(response: Response): Promise<ErrorEnvelope | undefined> {
  const contentType = response.headers.get('content-type') ?? ''
  if (!contentType.includes('application/json')) {
    return undefined
  }

  try {
    return (await response.clone().json()) as ErrorEnvelope
  } catch {
    return undefined
  }
}

function shouldRefreshSession(
  error: unknown,
  path: string,
  options: InternalApiRequestOptions,
): boolean {
  if (!(error instanceof HTTPError)) {
    return false
  }

  if (options.authMode === 'none' || options.hasRetried || options.skipAuthRefresh) {
    return false
  }

  if (AUTH_RECOVERY_PATHS.has(path)) {
    return false
  }

  return error.response.status === 401
}

async function refreshSession(client: KyInstance, baseUrl: string): Promise<void> {
  await client(buildUrl(baseUrl, 'auth/refresh'), {
    method: 'POST',
  })
}

function buildKyOptions(options: InternalApiRequestOptions): Options {
  return {
    headers: options.headers,
    json: options.json,
    method: options.method,
    searchParams: options.searchParams,
    signal: options.signal,
  }
}

function buildUrl(baseUrl: string, path: string): string {
  return `${baseUrl.replace(/\/+$/, '')}/${path.replace(/^\/+/, '')}`
}

function normalizePath(path: string): string {
  return path.replace(/^\/+/, '')
}

function resolveBaseUrl(baseUrl: string | undefined): string {
  if (baseUrl) {
    if (baseUrl.startsWith('/')) {
      return new URL(baseUrl, window.location.origin).toString().replace(/\/+$/, '')
    }

    return baseUrl
  }

  return new URL('/api', window.location.origin).toString().replace(/\/+$/, '')
}

function redirectToLogin(onAuthRedirect?: (href: string) => void) {
  const currentHref = `${window.location.pathname}${window.location.search}${window.location.hash}`
  if (window.location.pathname === '/login') {
    return
  }

  const redirectHref = `/login?redirect=${encodeURIComponent(currentHref)}`
  if (onAuthRedirect) {
    onAuthRedirect(redirectHref)
    return
  }

  window.location.assign(redirectHref)
}
