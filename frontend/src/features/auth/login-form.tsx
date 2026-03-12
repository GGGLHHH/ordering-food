import { useQueryClient } from '@tanstack/react-query'
import { Link } from '@tanstack/react-router'
import { ApiError } from '#/integrations/http'
import { refetchCurrentUser, useAuthSessionQuery, useLoginMutation } from './queries'

const SAFE_REDIRECT_FALLBACK = '/'

interface LoginFormProps {
  initialIdentifier?: string
  onSuccessRedirect: (href: string) => Promise<void> | void
  redirectTo?: string
}

export function LoginForm({ initialIdentifier, onSuccessRedirect, redirectTo }: LoginFormProps) {
  const queryClient = useQueryClient()
  const loginMutation = useLoginMutation()
  const sessionQuery = useAuthSessionQuery()
  const formError =
    loginMutation.error instanceof ApiError
      ? loginMutation.error.message
      : loginMutation.error
        ? '登录失败，请稍后重试。'
        : null

  async function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault()
    loginMutation.reset()

    const formData = new FormData(event.currentTarget)
    const identity_type = String(formData.get('identity_type') ?? 'email')
    const identifier = String(formData.get('identifier') ?? '').trim()
    const password = String(formData.get('password') ?? '')

    try {
      await loginMutation.mutateAsync({
        identifier,
        identity_type,
        password,
      })
      await refetchCurrentUser(queryClient)
      await onSuccessRedirect(normalizeRedirectTarget(redirectTo))
    } catch {
      return
    }
  }

  const currentUser = sessionQuery.data

  return (
    <section className="island-shell rise-in relative overflow-hidden rounded-[2rem] px-6 py-8 sm:px-8 sm:py-10">
      <div className="pointer-events-none absolute -top-16 right-0 h-40 w-40 rounded-full bg-[radial-gradient(circle,rgba(79,184,178,0.24),transparent_70%)]" />
      <p className="island-kicker mb-3">Account</p>
      <h1 className="mb-3 font-bold text-3xl text-[var(--sea-ink)] tracking-tight sm:text-4xl">
        登录你的账户
      </h1>
      <p className="mb-6 max-w-xl text-[var(--sea-ink-soft)] text-sm sm:text-base">
        使用后端签发的 HttpOnly Cookie
        建立会话。登录成功后，我们会重新拉取当前用户信息，再跳回原始页面。
      </p>

      {currentUser ? (
        <div className="mb-5 rounded-2xl border border-[rgba(47,106,74,0.18)] bg-[rgba(79,184,178,0.1)] px-4 py-3 text-[var(--sea-ink)] text-sm">
          当前已登录为 <strong>{currentUser.display_name}</strong>。
        </div>
      ) : null}

      <form className="grid gap-4" onSubmit={handleSubmit}>
        <label className="grid gap-2">
          <span className="font-semibold text-[var(--sea-ink)] text-sm">身份类型</span>
          <select
            name="identity_type"
            defaultValue="email"
            className="h-11 rounded-2xl border border-[var(--line)] bg-white/70 px-4 text-[var(--sea-ink)] outline-none transition focus:border-[rgba(47,106,74,0.4)]"
          >
            <option value="email">邮箱</option>
            <option value="phone">手机号</option>
          </select>
        </label>

        <label className="grid gap-2">
          <span className="font-semibold text-[var(--sea-ink)] text-sm">账号</span>
          <input
            name="identifier"
            type="text"
            autoComplete="username"
            defaultValue={initialIdentifier}
            required
            className="h-11 rounded-2xl border border-[var(--line)] bg-white/70 px-4 text-[var(--sea-ink)] outline-none transition focus:border-[rgba(47,106,74,0.4)]"
          />
        </label>

        <label className="grid gap-2">
          <span className="font-semibold text-[var(--sea-ink)] text-sm">密码</span>
          <input
            name="password"
            type="password"
            autoComplete="current-password"
            required
            className="h-11 rounded-2xl border border-[var(--line)] bg-white/70 px-4 text-[var(--sea-ink)] outline-none transition focus:border-[rgba(47,106,74,0.4)]"
          />
        </label>

        {formError ? (
          <p
            role="alert"
            className="rounded-2xl border border-[rgba(190,74,65,0.24)] bg-[rgba(190,74,65,0.1)] px-4 py-3 text-[var(--danger,#9f3a36)] text-sm"
          >
            {formError}
          </p>
        ) : null}

        <button
          type="submit"
          disabled={loginMutation.isPending}
          className="mt-2 inline-flex h-11 items-center justify-center rounded-full border border-[rgba(50,143,151,0.3)] bg-[rgba(79,184,178,0.18)] px-5 font-semibold text-[var(--lagoon-deep)] text-sm transition hover:-translate-y-0.5 hover:bg-[rgba(79,184,178,0.26)] disabled:cursor-not-allowed disabled:opacity-60"
        >
          {loginMutation.isPending ? '登录中...' : '登录'}
        </button>
      </form>

      <p className="mt-5 text-[var(--sea-ink-soft)] text-sm">
        还没有账号？
        <Link
          to="/register"
          search={buildAuthSearch(redirectTo)}
          className="ml-1 font-semibold text-[var(--lagoon-deep)] no-underline transition hover:opacity-80"
        >
          去注册
        </Link>
      </p>
    </section>
  )
}

function buildAuthSearch(redirectTo?: string) {
  return redirectTo ? { redirect: redirectTo } : undefined
}

function normalizeRedirectTarget(redirectTo?: string) {
  if (!redirectTo || !redirectTo.startsWith('/') || redirectTo.startsWith('//')) {
    return SAFE_REDIRECT_FALLBACK
  }

  return redirectTo
}
