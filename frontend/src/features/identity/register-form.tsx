import { Link } from '@tanstack/react-router'

import { ApiError } from '#/integrations/http'

import { useCreateIdentityUserMutation } from './queries'

interface RegisterFormProps {
  onSuccessRedirect: (identifier: string) => Promise<void> | void
  redirectTo?: string
}

export function RegisterForm({ onSuccessRedirect, redirectTo }: RegisterFormProps) {
  const registerMutation = useCreateIdentityUserMutation()
  const formError =
    registerMutation.error instanceof ApiError
      ? registerMutation.error.message
      : registerMutation.error
        ? '注册失败，请稍后重试。'
        : null
  const loginSearch = buildAuthSearch(redirectTo)

  async function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault()
    registerMutation.reset()

    const formData = new FormData(event.currentTarget)
    const displayName = String(formData.get('display_name') ?? '').trim()
    const identityType = String(formData.get('identity_type') ?? 'email')
    const identifier = String(formData.get('identifier') ?? '').trim()
    const password = String(formData.get('password') ?? '')

    try {
      await registerMutation.mutateAsync({
        display_name: displayName,
        identities: [
          {
            identifier,
            identity_type: identityType,
          },
        ],
        password,
      })
      await onSuccessRedirect(identifier)
    } catch {
      return
    }
  }

  return (
    <section className="island-shell rise-in relative overflow-hidden rounded-[2rem] px-6 py-8 sm:px-8 sm:py-10">
      <div className="pointer-events-none absolute top-0 right-0 h-44 w-44 rounded-full bg-[radial-gradient(circle,rgba(47,106,74,0.18),transparent_72%)]" />
      <p className="island-kicker mb-3">Account</p>
      <h1 className="mb-3 text-3xl font-bold tracking-tight text-[var(--sea-ink)] sm:text-4xl">
        创建一个新账户
      </h1>
      <p className="mb-6 max-w-xl text-sm text-[var(--sea-ink-soft)] sm:text-base">
        这是一个最小注册页。提交后会调用 `POST /api/identity/users`
        创建用户，然后跳回登录页继续完成会话建立。
      </p>

      <form className="grid gap-4" onSubmit={handleSubmit}>
        <label className="grid gap-2">
          <span className="text-sm font-semibold text-[var(--sea-ink)]">显示名称</span>
          <input
            name="display_name"
            type="text"
            autoComplete="nickname"
            required
            className="h-11 rounded-2xl border border-[var(--line)] bg-white/70 px-4 text-[var(--sea-ink)] transition outline-none focus:border-[rgba(47,106,74,0.4)]"
          />
        </label>

        <label className="grid gap-2">
          <span className="text-sm font-semibold text-[var(--sea-ink)]">身份类型</span>
          <select
            name="identity_type"
            defaultValue="email"
            className="h-11 rounded-2xl border border-[var(--line)] bg-white/70 px-4 text-[var(--sea-ink)] transition outline-none focus:border-[rgba(47,106,74,0.4)]"
          >
            <option value="email">邮箱</option>
            <option value="phone">手机号</option>
          </select>
        </label>

        <label className="grid gap-2">
          <span className="text-sm font-semibold text-[var(--sea-ink)]">账号</span>
          <input
            name="identifier"
            type="text"
            autoComplete="username"
            required
            className="h-11 rounded-2xl border border-[var(--line)] bg-white/70 px-4 text-[var(--sea-ink)] transition outline-none focus:border-[rgba(47,106,74,0.4)]"
          />
        </label>

        <label className="grid gap-2">
          <span className="text-sm font-semibold text-[var(--sea-ink)]">密码</span>
          <input
            name="password"
            type="password"
            autoComplete="new-password"
            required
            className="h-11 rounded-2xl border border-[var(--line)] bg-white/70 px-4 text-[var(--sea-ink)] transition outline-none focus:border-[rgba(47,106,74,0.4)]"
          />
        </label>

        {formError ? (
          <p
            role="alert"
            className="rounded-2xl border border-[rgba(190,74,65,0.24)] bg-[rgba(190,74,65,0.1)] px-4 py-3 text-sm text-[var(--danger,#9f3a36)]"
          >
            {formError}
          </p>
        ) : null}

        <button
          type="submit"
          disabled={registerMutation.isPending}
          className="mt-2 inline-flex h-11 items-center justify-center rounded-full border border-[rgba(47,106,74,0.24)] bg-[rgba(47,106,74,0.14)] px-5 text-sm font-semibold text-[var(--sea-ink)] transition hover:-translate-y-0.5 hover:bg-[rgba(47,106,74,0.2)] disabled:cursor-not-allowed disabled:opacity-60"
        >
          {registerMutation.isPending ? '注册中...' : '注册'}
        </button>
      </form>

      <p className="mt-5 text-sm text-[var(--sea-ink-soft)]">
        已有账号？
        <Link
          to="/login"
          search={loginSearch}
          className="ml-1 font-semibold text-[var(--lagoon-deep)] no-underline transition hover:opacity-80"
        >
          去登录
        </Link>
      </p>
    </section>
  )
}

function buildAuthSearch(redirectTo?: string) {
  return redirectTo ? { redirect: redirectTo } : undefined
}
