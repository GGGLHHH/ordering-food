import { Link, useRouterState } from '@tanstack/react-router'
import { useAuthSessionQuery, useLogoutMutation } from '#/features/auth/queries'
import ThemeToggle from './ThemeToggle'

export default function Header() {
  const location = useRouterState({
    select: (state) => state.location,
  })
  const logoutMutation = useLogoutMutation()
  const sessionQuery = useAuthSessionQuery()
  const currentUser = sessionQuery.data
  const authEntrySearch =
    location.pathname === '/login' || location.pathname === '/register'
      ? undefined
      : { redirect: location.href }

  return (
    <header className="sticky top-0 z-50 border-[var(--line)] border-b bg-[var(--header-bg)] px-4 backdrop-blur-lg">
      <nav className="page-wrap flex flex-wrap items-center gap-x-3 gap-y-2 py-3 sm:py-4">
        <h2 className="m-0 flex-shrink-0 font-semibold text-base tracking-tight">
          <Link
            to="/"
            className="inline-flex items-center gap-2 rounded-full border border-[var(--chip-line)] bg-[var(--chip-bg)] px-3 py-1.5 text-[var(--sea-ink)] text-sm no-underline shadow-[0_8px_24px_rgba(30,90,72,0.08)] sm:px-4 sm:py-2"
          >
            <span className="h-2 w-2 rounded-full bg-[linear-gradient(90deg,#56c6be,#7ed3bf)]" />
            Ordering Food
          </Link>
        </h2>

        <div className="ml-auto flex items-center gap-1.5 sm:ml-0 sm:gap-2">
          {currentUser ? (
            <div className="inline-flex items-center gap-2 rounded-full border border-[rgba(50,143,151,0.22)] bg-[rgba(79,184,178,0.12)] px-3 py-1.5 text-[var(--sea-ink)] text-sm shadow-[0_8px_24px_rgba(30,90,72,0.08)]">
              <span className="font-semibold">{currentUser.display_name}</span>
              <button
                type="button"
                onClick={() => {
                  void logoutMutation.mutateAsync()
                }}
                disabled={logoutMutation.isPending}
                className="rounded-full border border-[rgba(50,143,151,0.2)] bg-white/70 px-2.5 py-1 font-semibold text-[var(--lagoon-deep)] text-xs transition hover:bg-white disabled:cursor-not-allowed disabled:opacity-60"
              >
                {logoutMutation.isPending ? '退出中...' : '退出'}
              </button>
            </div>
          ) : (
            <div className="flex items-center gap-1.5 sm:gap-2">
              <Link
                to="/login"
                search={authEntrySearch}
                className="rounded-full border border-[rgba(50,143,151,0.24)] bg-[rgba(79,184,178,0.12)] px-3 py-1.5 font-semibold text-[var(--lagoon-deep)] text-sm no-underline shadow-[0_8px_24px_rgba(30,90,72,0.08)] transition hover:-translate-y-0.5 hover:bg-[rgba(79,184,178,0.2)]"
              >
                登录
              </Link>
              <Link
                to="/register"
                search={authEntrySearch}
                className="rounded-full border border-[rgba(47,106,74,0.22)] bg-[rgba(47,106,74,0.12)] px-3 py-1.5 font-semibold text-[var(--sea-ink)] text-sm no-underline shadow-[0_8px_24px_rgba(30,90,72,0.08)] transition hover:-translate-y-0.5 hover:bg-[rgba(47,106,74,0.18)]"
              >
                注册
              </Link>
            </div>
          )}
          <ThemeToggle />
        </div>

        <div className="order-3 flex w-full flex-wrap items-center gap-x-4 gap-y-1 pb-1 font-semibold text-sm sm:order-2 sm:w-auto sm:flex-nowrap sm:pb-0">
          <Link to="/" className="nav-link" activeProps={{ className: 'nav-link is-active' }}>
            Home
          </Link>
          <Link to="/menu" className="nav-link" activeProps={{ className: 'nav-link is-active' }}>
            Menu
          </Link>
          <Link to="/about" className="nav-link" activeProps={{ className: 'nav-link is-active' }}>
            About
          </Link>
          <Link
            to="/login"
            search={authEntrySearch}
            className="nav-link"
            activeProps={{ className: 'nav-link is-active' }}
          >
            Account
          </Link>
        </div>
      </nav>
    </header>
  )
}
