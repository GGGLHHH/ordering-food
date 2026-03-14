import { Link } from '@tanstack/react-router'

import { ApiError } from '#/integrations/http'

import { useOrderQuery } from './queries'

interface OrderPageProps {
  orderId: string
}

export function OrderPage({ orderId }: OrderPageProps) {
  const orderQuery = useOrderQuery(orderId)
  const order = orderQuery.data
  const statusSteps = [
    'pending_acceptance',
    'accepted',
    'preparing',
    'ready_for_pickup',
    'completed',
  ]
  const terminalTone =
    order?.status === 'cancelled_by_customer' || order?.status === 'rejected_by_store'
      ? 'danger'
      : 'neutral'

  if (orderQuery.isPending) {
    return (
      <main className="page-wrap px-4 pt-14 pb-10">
        <StateCard
          title="Fetching order"
          message="Loading the latest order snapshot from the API."
        />
      </main>
    )
  }

  if (orderQuery.error) {
    return (
      <main className="page-wrap px-4 pt-14 pb-10">
        <StateCard
          title="Order unavailable"
          message={formatOrderError(orderQuery.error)}
          tone="danger"
        />
      </main>
    )
  }

  if (!order) {
    return (
      <main className="page-wrap px-4 pt-14 pb-10">
        <StateCard
          title="Order not found"
          message="We could not find this order. It may have expired or belong to another account."
          tone="danger"
        />
      </main>
    )
  }

  return (
    <main className="page-wrap px-4 pt-14 pb-10">
      <section className="island-shell rise-in relative overflow-hidden rounded-[2rem] px-6 py-8 sm:px-10 sm:py-10">
        <div className="pointer-events-none absolute -top-18 right-0 h-48 w-48 rounded-full bg-[radial-gradient(circle,rgba(79,184,178,0.26),transparent_70%)]" />
        <p className="island-kicker mb-3">Order Detail</p>
        <div className="flex flex-col gap-3 lg:flex-row lg:items-end lg:justify-between">
          <div className="space-y-3">
            <h1 className="display-title text-4xl leading-[1.02] font-bold tracking-tight text-[var(--sea-ink)] sm:text-5xl">
              {humanizeStatus(order.status)}
            </h1>
            <p className="max-w-2xl text-sm text-[var(--sea-ink-soft)] sm:text-base">
              This order page is powered by the real order context. It renders the persisted
              snapshot instead of rebuilding the basket from the menu.
            </p>
            <div className="flex flex-wrap gap-3">
              <Link
                to="/orders"
                className="rounded-full border border-[rgba(50,143,151,0.24)] bg-[rgba(79,184,178,0.12)] px-4 py-2 text-sm font-semibold text-[var(--lagoon-deep)] no-underline transition hover:-translate-y-0.5 hover:bg-[rgba(79,184,178,0.2)]"
              >
                Back to orders
              </Link>
              <Link
                to="/menu"
                className="rounded-full border border-[rgba(47,106,74,0.22)] bg-[rgba(47,106,74,0.1)] px-4 py-2 text-sm font-semibold text-[var(--sea-ink)] no-underline transition hover:-translate-y-0.5 hover:bg-[rgba(47,106,74,0.16)]"
              >
                Back to menu
              </Link>
            </div>
          </div>

          <dl className="grid gap-3 rounded-[1.5rem] border border-[rgba(50,143,151,0.18)] bg-white/55 px-4 py-4 text-sm sm:min-w-[320px]">
            <div className="flex items-center justify-between gap-4">
              <dt className="text-[var(--sea-ink-soft)]">Order ID</dt>
              <dd className="max-w-[180px] truncate font-semibold text-[var(--sea-ink)]">
                {order.order_id}
              </dd>
            </div>
            <div className="flex items-center justify-between gap-4">
              <dt className="text-[var(--sea-ink-soft)]">Store</dt>
              <dd className="font-semibold text-[var(--sea-ink)]">{order.store_id}</dd>
            </div>
            <div className="flex items-center justify-between gap-4">
              <dt className="text-[var(--sea-ink-soft)]">Updated</dt>
              <dd className="font-semibold text-[var(--sea-ink)]">
                {formatTimestamp(order.updated_at)}
              </dd>
            </div>
          </dl>
        </div>
      </section>

      <section className="mt-8 grid gap-8 lg:grid-cols-[minmax(0,1fr)_320px]">
        <div className="space-y-4">
          <div className="island-shell rounded-[1.6rem] p-5">
            <p className="island-kicker mb-3">Timeline</p>
            <div className="grid gap-3">
              {statusSteps.map((step) => {
                const isActive = step === order.status
                const isDone = statusSteps.indexOf(step) <= statusSteps.indexOf(order.status)

                return (
                  <div
                    key={step}
                    className={
                      isActive
                        ? 'rounded-[1rem] border border-[rgba(50,143,151,0.28)] bg-[rgba(79,184,178,0.14)] px-4 py-3'
                        : isDone
                          ? 'rounded-[1rem] border border-[rgba(47,106,74,0.18)] bg-[rgba(47,106,74,0.08)] px-4 py-3'
                          : 'rounded-[1rem] border border-[var(--line)] bg-white/45 px-4 py-3'
                    }
                  >
                    <div className="flex items-center justify-between gap-3">
                      <span className="font-semibold text-[var(--sea-ink)]">
                        {humanizeStatus(step)}
                      </span>
                      <span className="text-xs tracking-[0.18em] text-[var(--sea-ink-soft)] uppercase">
                        {isActive ? 'Current' : isDone ? 'Done' : 'Pending'}
                      </span>
                    </div>
                  </div>
                )
              })}

              {order.status === 'cancelled_by_customer' || order.status === 'rejected_by_store' ? (
                <div
                  className={
                    terminalTone === 'danger'
                      ? 'rounded-[1rem] border border-[rgba(190,74,65,0.22)] bg-[rgba(190,74,65,0.08)] px-4 py-3'
                      : 'rounded-[1rem] border border-[var(--line)] bg-white/45 px-4 py-3'
                  }
                >
                  <div className="flex items-center justify-between gap-3">
                    <span className="font-semibold text-[var(--sea-ink)]">
                      {humanizeStatus(order.status)}
                    </span>
                    <span className="text-xs tracking-[0.18em] text-[var(--sea-ink-soft)] uppercase">
                      Terminal
                    </span>
                  </div>
                </div>
              ) : null}
            </div>
          </div>

          {order.items.map((item, index) => (
            <article
              key={`${item.line_number}-${item.menu_item_id}`}
              className="island-shell rise-in rounded-[1.6rem] p-5"
              style={{ animationDelay: `${index * 60 + 60}ms` }}
            >
              <div className="mb-4 flex items-start justify-between gap-3">
                <div>
                  <p className="mb-2 text-xs tracking-[0.18em] text-[var(--sea-ink-soft)] uppercase">
                    Line {item.line_number}
                  </p>
                  <h2 className="text-xl font-semibold text-[var(--sea-ink)]">{item.name}</h2>
                </div>
                <p className="rounded-full border border-[rgba(47,106,74,0.18)] bg-[rgba(47,106,74,0.12)] px-3 py-1 text-sm font-semibold text-[var(--sea-ink)]">
                  {formatMoney(item.line_total_amount)}
                </p>
              </div>

              <div className="grid gap-2 text-sm text-[var(--sea-ink-soft)] sm:grid-cols-3">
                <p className="m-0">Menu item: {item.menu_item_id}</p>
                <p className="m-0">Unit price: {formatMoney(item.unit_price_amount)}</p>
                <p className="m-0">Quantity: {item.quantity}</p>
              </div>
            </article>
          ))}
        </div>

        <aside className="space-y-4">
          <div className="island-shell rounded-[1.6rem] p-5">
            <p className="island-kicker mb-3">Summary</p>
            <div className="space-y-3 text-sm">
              <div className="flex items-center justify-between gap-3">
                <span className="text-[var(--sea-ink-soft)]">Customer</span>
                <span className="max-w-[180px] truncate font-semibold text-[var(--sea-ink)]">
                  {order.customer_id}
                </span>
              </div>
              <div className="flex items-center justify-between gap-3">
                <span className="text-[var(--sea-ink-soft)]">Subtotal</span>
                <span className="font-semibold text-[var(--sea-ink)]">
                  {formatMoney(order.subtotal_amount)}
                </span>
              </div>
              <div className="flex items-center justify-between gap-3">
                <span className="text-[var(--sea-ink-soft)]">Total</span>
                <span className="font-semibold text-[var(--sea-ink)]">
                  {formatMoney(order.total_amount)}
                </span>
              </div>
            </div>
          </div>

          <div className="island-shell rounded-[1.6rem] p-5">
            <p className="island-kicker mb-3">Next action</p>
            <div className="flex flex-col gap-3">
              <button
                type="button"
                onClick={() => {
                  void orderQuery.refetch()
                }}
                className="rounded-full border border-[rgba(50,143,151,0.24)] bg-[rgba(79,184,178,0.12)] px-4 py-2 text-sm font-semibold text-[var(--lagoon-deep)] transition hover:-translate-y-0.5 hover:bg-[rgba(79,184,178,0.2)]"
              >
                Refresh status
              </button>
            </div>
          </div>
        </aside>
      </section>
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

  return 'The order request failed. Please refresh or try again later.'
}
