import { createFileRoute, Link } from '@tanstack/react-router'

export const Route = createFileRoute('/')({ component: App })

function App() {
  return (
    <main className="page-wrap px-4 pt-14 pb-8">
      <section className="island-shell rise-in relative overflow-hidden rounded-[2rem] px-6 py-10 sm:px-10 sm:py-14">
        <div className="pointer-events-none absolute -top-24 -left-20 h-56 w-56 rounded-full bg-[radial-gradient(circle,rgba(79,184,178,0.32),transparent_66%)]" />
        <div className="pointer-events-none absolute -right-20 -bottom-20 h-56 w-56 rounded-full bg-[radial-gradient(circle,rgba(47,106,74,0.18),transparent_66%)]" />
        <p className="island-kicker mb-3">Ordering Food</p>
        <h1 className="display-title mb-5 max-w-3xl text-4xl leading-[1.02] font-bold tracking-tight text-[var(--sea-ink)] sm:text-6xl">
          A clean shell for real ordering flows.
        </h1>
        <p className="mb-8 max-w-2xl text-base text-[var(--sea-ink-soft)] sm:text-lg">
          The starter copy and demo navigation are gone. This landing page now leaves room for menu
          browsing, cart management, checkout, and order tracking.
        </p>
        <div className="flex flex-wrap gap-3">
          <Link
            to="/menu"
            className="rounded-full border border-[rgba(50,143,151,0.3)] bg-[rgba(79,184,178,0.14)] px-5 py-2.5 text-sm font-semibold text-[var(--lagoon-deep)] no-underline transition hover:-translate-y-0.5 hover:bg-[rgba(79,184,178,0.24)]"
          >
            Browse Menu
          </Link>
          <Link
            to="/about"
            className="rounded-full border border-[rgba(47,106,74,0.24)] bg-[rgba(47,106,74,0.12)] px-5 py-2.5 text-sm font-semibold text-[var(--sea-ink)] no-underline transition hover:-translate-y-0.5 hover:bg-[rgba(47,106,74,0.2)]"
          >
            Project Overview
          </Link>
        </div>
      </section>

      <section className="mt-8 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        {[
          [
            'Menu Discovery',
            'Use the homepage to surface categories, items, and merchant context.',
          ],
          ['Cart State', 'Keep quantity changes, notes, and pricing logic close to the UI flow.'],
          ['Checkout', 'Reserve space for address, payment, and confirmation steps.'],
          ['Order Tracking', 'Add post-purchase status updates without starter noise in the way.'],
        ].map(([title, desc], index) => (
          <article
            key={title}
            className="island-shell feature-card rise-in rounded-2xl p-5"
            style={{ animationDelay: `${index * 90 + 80}ms` }}
          >
            <h2 className="mb-2 text-base font-semibold text-[var(--sea-ink)]">{title}</h2>
            <p className="m-0 text-sm text-[var(--sea-ink-soft)]">{desc}</p>
          </article>
        ))}
      </section>

      <section className="island-shell mt-8 rounded-2xl p-6">
        <p className="island-kicker mb-2">Next Moves</p>
        <ul className="m-0 list-disc space-y-2 pl-5 text-sm text-[var(--sea-ink-soft)]">
          <li>
            Replace <code>src/routes/index.tsx</code> with your first real customer-facing flow.
          </li>
          <li>
            Expand <code>src/routes</code> with menu, cart, checkout, and order detail pages.
          </li>
          <li>Wire generated API clients and shared tokens before layering business state.</li>
        </ul>
      </section>
    </main>
  )
}
