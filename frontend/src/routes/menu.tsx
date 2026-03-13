import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { z } from 'zod'
import { MenuPage } from '#/features/menu/menu-page'

const menuSearchSchema = z.object({
  category: z.string().optional(),
})

export const Route = createFileRoute('/menu')({
  component: MenuRoute,
  validateSearch: menuSearchSchema,
})

function MenuRoute() {
  const navigate = useNavigate()
  const { category } = Route.useSearch()

  return (
    <MenuPage
      selectedCategorySlug={category}
      onCategoryChange={async (slug) => {
        await navigate({
          search: slug
            ? {
                category: slug,
              }
            : {},
          to: '/menu',
        })
      }}
    />
  )
}
