import { useDeferredValue } from 'react'

import { ApiError } from '#/integrations/http'
import { cn } from '#/lib/utils'

import { useMenuCategoriesQuery, useMenuItemsQuery, useMenuStoreQuery } from './queries'

interface MenuPageProps {
  onCategoryChange: (slug?: string) => void
  selectedCategorySlug?: string
}

export function MenuPage({ onCategoryChange, selectedCategorySlug }: MenuPageProps) {
  const deferredCategorySlug = useDeferredValue(selectedCategorySlug)
  const storeQuery = useMenuStoreQuery()
  const categoriesQuery = useMenuCategoriesQuery()
  const itemsQuery = useMenuItemsQuery(deferredCategorySlug)

  const store = storeQuery.data
  const categories = categoriesQuery.data?.categories ?? []
  const items = itemsQuery.data?.items ?? []
  const activeCategorySlug = deferredCategorySlug?.trim() || undefined
  const activeCategory = activeCategorySlug
    ? categories.find((category) => category.slug === activeCategorySlug)
    : undefined
  const isLoading = storeQuery.isPending || categoriesQuery.isPending || itemsQuery.isPending
  const hasError = storeQuery.error || categoriesQuery.error || itemsQuery.error

  return (
    <main className="page-wrap px-4 pt-14 pb-10">
      <section className="island-shell rise-in relative overflow-hidden rounded-[2rem] px-6 py-8 sm:px-10 sm:py-10">
        <div className="pointer-events-none absolute -top-20 right-0 h-52 w-52 rounded-full bg-[radial-gradient(circle,rgba(79,184,178,0.26),transparent_70%)]" />
        <div className="pointer-events-none absolute -bottom-20 left-0 h-52 w-52 rounded-full bg-[radial-gradient(circle,rgba(47,106,74,0.18),transparent_70%)]" />
        <p className="island-kicker mb-3">Menu</p>
        <div className="flex flex-col gap-3 lg:flex-row lg:items-end lg:justify-between">
          <div className="space-y-3">
            <h1 className="display-title max-w-3xl text-4xl leading-[1.02] font-bold tracking-tight text-[var(--sea-ink)] sm:text-5xl">
              {store?.name ?? 'Loading today’s menu...'}
            </h1>
            <p className="max-w-2xl text-sm text-[var(--sea-ink-soft)] sm:text-base">
              Browse a single-store menu powered by the new `menu` context. Categories, item
              filters, and prices now come from the API instead of placeholder copy.
            </p>
          </div>

          <dl className="grid gap-3 rounded-[1.5rem] border border-[rgba(50,143,151,0.18)] bg-white/55 px-4 py-4 text-sm sm:min-w-[280px]">
            <div className="flex items-center justify-between gap-4">
              <dt className="text-[var(--sea-ink-soft)]">Currency</dt>
              <dd className="font-semibold text-[var(--sea-ink)]">
                {store?.currency_code ?? '--'}
              </dd>
            </div>
            <div className="flex items-center justify-between gap-4">
              <dt className="text-[var(--sea-ink-soft)]">Timezone</dt>
              <dd className="font-semibold text-[var(--sea-ink)]">{store?.timezone ?? '--'}</dd>
            </div>
            <div className="flex items-center justify-between gap-4">
              <dt className="text-[var(--sea-ink-soft)]">Visible categories</dt>
              <dd className="font-semibold text-[var(--sea-ink)]">{categories.length}</dd>
            </div>
          </dl>
        </div>
      </section>

      <section className="mt-8 grid gap-8 lg:grid-cols-[280px_minmax(0,1fr)]">
        <aside className="space-y-4">
          <div className="island-shell rounded-[1.75rem] p-5">
            <p className="island-kicker mb-3">Browse</p>
            <div className="flex flex-wrap gap-2 lg:flex-col">
              <FilterButton
                isActive={!activeCategorySlug}
                label="All items"
                onClick={() => {
                  onCategoryChange(undefined)
                }}
              />
              {categories.map((category) => (
                <FilterButton
                  key={category.category_id}
                  description={category.description}
                  isActive={category.slug === activeCategorySlug}
                  label={category.name}
                  onClick={() => {
                    onCategoryChange(category.slug)
                  }}
                />
              ))}
            </div>
          </div>

          <div className="island-shell rounded-[1.75rem] p-5">
            <p className="island-kicker mb-2">Selection</p>
            <h2 className="mb-2 text-lg font-semibold text-[var(--sea-ink)]">
              {activeCategory?.name ?? 'Everything on the menu'}
            </h2>
            <p className="m-0 text-sm text-[var(--sea-ink-soft)]">
              {activeCategory?.description ??
                'Choose a category to narrow the list, or stay on all items to inspect the full menu.'}
            </p>
          </div>
        </aside>

        <section className="space-y-4">
          {isLoading ? (
            <StateCard
              message="Loading categories and items from the menu API."
              title="Fetching menu"
            />
          ) : null}

          {hasError ? (
            <StateCard message={formatMenuError(hasError)} title="Menu unavailable" tone="danger" />
          ) : null}

          {!isLoading && !hasError && items.length === 0 ? (
            <StateCard
              message="No items matched this filter. Try another category to verify the API response."
              title="No matching items"
            />
          ) : null}

          {!isLoading && !hasError && items.length > 0 ? (
            <>
              <div className="flex items-center justify-between gap-4">
                <div>
                  <p className="island-kicker mb-1">Live inventory</p>
                  <h2 className="text-2xl font-semibold text-[var(--sea-ink)]">
                    {items.length} item{items.length === 1 ? '' : 's'}
                  </h2>
                </div>
                <p className="max-w-sm text-right text-sm text-[var(--sea-ink-soft)]">
                  Sorted by the backend `sort_order`, then rendered as responsive cards for mobile
                  and desktop.
                </p>
              </div>

              <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
                {items.map((item, index) => (
                  <article
                    key={item.item_id}
                    className="island-shell rise-in flex min-h-[240px] flex-col rounded-[1.75rem] p-5"
                    style={{ animationDelay: `${index * 45 + 60}ms` }}
                  >
                    <div className="mb-4 flex items-start justify-between gap-3">
                      <div>
                        <p className="mb-2 inline-flex rounded-full border border-[rgba(50,143,151,0.18)] bg-[rgba(79,184,178,0.1)] px-2.5 py-1 text-xs font-semibold tracking-[0.18em] text-[var(--lagoon-deep)] uppercase">
                          {resolveCategoryName(item.category_id, categories)}
                        </p>
                        <h3 className="text-xl font-semibold text-[var(--sea-ink)]">{item.name}</h3>
                      </div>
                      <p className="rounded-full border border-[rgba(47,106,74,0.18)] bg-[rgba(47,106,74,0.12)] px-3 py-1 text-sm font-semibold text-[var(--sea-ink)]">
                        {formatMenuPrice(item.price_amount, store?.currency_code)}
                      </p>
                    </div>

                    <p className="mb-5 flex-1 text-sm leading-6 text-[var(--sea-ink-soft)]">
                      {item.description ??
                        'No description yet. The API returned this item without extra copy.'}
                    </p>

                    <div className="flex items-center justify-between gap-3 border-t border-[var(--line)] pt-4">
                      <span className="text-xs tracking-[0.18em] text-[var(--sea-ink-soft)] uppercase">
                        {item.slug}
                      </span>
                      <span className="text-xs font-semibold tracking-[0.16em] text-[var(--lagoon-deep)] uppercase">
                        {item.status}
                      </span>
                    </div>
                  </article>
                ))}
              </div>
            </>
          ) : null}
        </section>
      </section>
    </main>
  )
}

function FilterButton({
  description,
  isActive,
  label,
  onClick,
}: {
  description?: string
  isActive: boolean
  label: string
  onClick: () => void
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={cn(
        'group w-full rounded-[1.2rem] border px-4 py-3 text-left transition',
        isActive
          ? 'border-[rgba(50,143,151,0.28)] bg-[rgba(79,184,178,0.16)] shadow-[0_12px_30px_rgba(50,143,151,0.12)]'
          : 'border-[var(--line)] bg-white/50 hover:border-[rgba(50,143,151,0.18)] hover:bg-white/80',
      )}
    >
      <div className="flex items-center justify-between gap-3">
        <span className="text-sm font-semibold text-[var(--sea-ink)]">{label}</span>
        <span
          className={cn(
            'h-2.5 w-2.5 rounded-full transition',
            isActive ? 'bg-[var(--lagoon-deep)]' : 'bg-[rgba(50,143,151,0.2)]',
          )}
        />
      </div>
      {description ? (
        <p className="mt-2 text-xs leading-5 text-[var(--sea-ink-soft)]">{description}</p>
      ) : null}
    </button>
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
      className={cn(
        'island-shell rounded-[1.75rem] p-6',
        tone === 'danger' ? 'border-[rgba(190,74,65,0.2)] bg-[rgba(190,74,65,0.08)]' : undefined,
      )}
    >
      <p className="island-kicker mb-2">{title}</p>
      <p className="m-0 text-sm leading-6 text-[var(--sea-ink-soft)]">{message}</p>
    </div>
  )
}

function formatMenuError(error: unknown) {
  if (error instanceof ApiError) {
    return error.message
  }

  return 'The menu request failed. Please refresh or verify that the backend is running.'
}

function formatMenuPrice(amount: number, currencyCode = 'CNY') {
  return new Intl.NumberFormat('zh-CN', {
    currency: currencyCode,
    style: 'currency',
  }).format(amount / 100)
}

function resolveCategoryName(
  categoryId: string,
  categories: Array<{ category_id: string; name: string }>,
) {
  return categories.find((category) => category.category_id === categoryId)?.name ?? 'Menu item'
}
