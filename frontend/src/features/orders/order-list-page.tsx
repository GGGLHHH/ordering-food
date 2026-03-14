import { Link } from '@tanstack/react-router'

import { ApiError } from '#/integrations/http'

import { useOrdersQuery } from './queries'

export function OrderListPage() {
  const ordersQuery = useOrdersQuery()
  const orders = ordersQuery.data?.orders ?? []

  if (ordersQuery.isPending) {
    return (
      <main className="page-wrap px-4 pt-14 pb-10">
        <StateCard title="Loading orders" message="Fetching your latest pickup orders." />
      </main>
    )
  }

  if (ordersQuery.error) {
    return (
      <main className="page-wrap px-4 pt-14 pb-10">
        <StateCard
          title="Orders unavailable"
          message={formatOrderError(ordersQuery.error)}
          tone="danger"
        />
      </main>
    )
  }

  return (
    <main className="page-wrap px-4 pt-14 pb-10">
      <section className="island-shell rise-in relative overflow-hidden rounded-[2rem] px-6 py-8 sm:px-10 sm:py-10">
        <div className="pointer-events-none absolute -top-18 right-0 h-48 w-48 rounded-full bg-[radial-gradient(circle,rgba(79,184,178,0.26),transparent_70%)]" />
        <p className="island-kicker mb-3">Order List</p>
        <div className="flex flex-col gap-3 lg:flex-row lg:items-end lg:justify-between">
          <div className="space-y-3">
            <h1 className="display-title text-4xl leading-[1.02] font-bold tracking-tight text-[var(--sea-ink)] sm:text-5xl">
              Your recent orders
            </h1>
            <p className="max-w-2xl text-sm text-[var(--sea-ink-soft)] sm:text-base">
              Review live order snapshots without rebuilding them from the menu. The list is sorted
              by the backend using the persisted creation time.
            </p>
          </div>

          <button
            type="button"
            onClick={() => {
              void ordersQuery.refetch()
            }}
            className="rounded-full border border-[rgba(50,143,151,0.24)] bg-[rgba(79,184,178,0.12)] px-4 py-2 text-sm font-semibold text-[var(--lagoon-deep)] transition hover:-translate-y-0.5 hover:bg-[rgba(79,184,178,0.2)]"
          >
            Refresh list
          </button>
        </div>
      </section>

      {orders.length === 0 ? (
        <section className="mt-8">
          <StateCard
            title="No orders yet"
            message="Place your first pickup order from the menu and it will appear here."
          />
        </section>
      ) : (
        <section className="mt-8 grid gap-4">
          {orders.map((order, index) => (
            <article
              key={order.order_id}
              className="island-shell rise-in rounded-[1.75rem] p-5"
              style={{ animationDelay: `${index * 60 + 40}ms` }}
            >
              <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
                <div className="space-y-2">
                  <p className="m-0 text-xs tracking-[0.18em] text-[var(--sea-ink-soft)] uppercase">
                    {order.order_id}
                  </p>
                  <h2 className="m-0 text-2xl font-semibold text-[var(--sea-ink)]">
                    {humanizeStatus(order.status)}
                  </h2>
                  <p className="m-0 text-sm text-[var(--sea-ink-soft)]">
                    {order.item_count} item{order.item_count === 1 ? '' : 's'} • updated{' '}
                    {formatTimestamp(order.updated_at)}
                  </p>
                  <p className="m-0 text-sm text-[var(--sea-ink-soft)]">
                    Detail page keeps the full snapshot and status timeline.
                  </p>
                </div>

                <div className="flex flex-col items-start gap-3 lg:items-end">
                  <p className="rounded-full border border-[rgba(47,106,74,0.18)] bg-[rgba(47,106,74,0.12)] px-3 py-1 text-sm font-semibold text-[var(--sea-ink)]">
                    {formatMoney(order.total_amount)}
                  </p>
                  <Link
                    params={{ orderId: order.order_id }}
                    to="/orders/$orderId"
                    className="rounded-full border border-[rgba(50,143,151,0.24)] bg-[rgba(79,184,178,0.12)] px-4 py-2 text-sm font-semibold text-[var(--lagoon-deep)] no-underline transition hover:-translate-y-0.5 hover:bg-[rgba(79,184,178,0.2)]"
                  >
                    Open detail page
                  </Link>
                </div>
              </div>
            </article>
          ))}
        </section>
      )}
    </main>
  )
}

function StateCard({
  message,
  title,
  tone = 'neutral',
}: {
  message: string
  title: string
  tone?: 'danger' | 'neutral'
}) {
  return (
    <div
      className={
        tone === 'danger'
          ? 'island-shell rounded-[1.75rem] border-[rgba(190,74,65,0.2)] bg-[rgba(190,74,65,0.08)] p-6'
          : 'island-shell rounded-[1.75rem] p-6'
      }
    >
      <p className="island-kicker mb-2">{title}</p>
      <p className="m-0 text-sm leading-6 text-[var(--sea-ink-soft)]">{message}</p>
    </div>
  )
}

function humanizeStatus(status: string) {
  return status
    .split('_')
    .map((segment) => segment.charAt(0).toUpperCase() + segment.slice(1))
    .join(' ')
}

function formatMoney(amount: number) {
  return new Intl.NumberFormat('zh-CN', {
    currency: 'CNY',
    style: 'currency',
  }).format(amount / 100)
}

function formatTimestamp(value: string) {
  const date = new Date(value)
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString('zh-CN')
}

function formatOrderError(error: unknown) {
  if (error instanceof ApiError) {
    return error.message
  }

  return 'The order list request failed. Please refresh or try again later.'
}
