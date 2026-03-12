import type { ErrorDetails, ErrorEnvelope } from '#/contracts/generated'

export interface ApiErrorInit {
  cause?: unknown
  code: string
  details?: ErrorDetails
  isRetryable: boolean
  message: string
  requestId?: string
  status: number | null
}

export class ApiError extends Error {
  cause?: unknown
  code: string
  details?: ErrorDetails
  isRetryable: boolean
  isUnauthorized: boolean
  requestId?: string
  status: number | null

  constructor({ cause, code, details, isRetryable, message, requestId, status }: ApiErrorInit) {
    super(message)

    this.name = 'ApiError'
    this.cause = cause
    this.code = code
    this.details = details
    this.isRetryable = isRetryable
    this.isUnauthorized = status === 401 || code === 'unauthorized'
    this.requestId = requestId
    this.status = status
  }
}

export function createHttpApiError(
  status: number,
  envelope?: Partial<ErrorEnvelope>,
  cause?: unknown,
): ApiError {
  return new ApiError({
    cause,
    code: envelope?.code ?? `http_${status}`,
    details: envelope?.details,
    isRetryable: status >= 500,
    message: envelope?.message ?? `Request failed with status ${status}`,
    requestId: envelope?.request_id,
    status,
  })
}

export function createNetworkApiError(cause?: unknown): ApiError {
  return new ApiError({
    cause,
    code: 'network_error',
    isRetryable: true,
    message: 'Network request failed',
    status: null,
  })
}

export function createTimeoutApiError(cause?: unknown): ApiError {
  return new ApiError({
    cause,
    code: 'timeout_error',
    isRetryable: true,
    message: 'Request timed out',
    status: null,
  })
}

export function createUnknownApiError(cause?: unknown): ApiError {
  const message = cause instanceof Error ? cause.message : 'Unknown request error'

  return new ApiError({
    cause,
    code: 'unknown_error',
    isRetryable: false,
    message,
    status: null,
  })
}

export function isAbortError(error: unknown): error is DOMException {
  return error instanceof DOMException && error.name === 'AbortError'
}
