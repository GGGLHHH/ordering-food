export function installMockLocalStorage() {
  const entries = new Map<string, string>()
  const originalDescriptor = Object.getOwnPropertyDescriptor(window, 'localStorage')

  const storage: Storage = {
    get length() {
      return entries.size
    },
    clear() {
      entries.clear()
    },
    getItem(key) {
      return entries.get(String(key)) ?? null
    },
    key(index) {
      return Array.from(entries.keys())[index] ?? null
    },
    removeItem(key) {
      entries.delete(String(key))
    },
    setItem(key, value) {
      entries.set(String(key), String(value))
    },
  }

  Object.defineProperty(window, 'localStorage', {
    configurable: true,
    value: storage,
  })

  return () => {
    if (originalDescriptor) {
      Object.defineProperty(window, 'localStorage', originalDescriptor)
      return
    }

    Reflect.deleteProperty(window, 'localStorage')
  }
}
