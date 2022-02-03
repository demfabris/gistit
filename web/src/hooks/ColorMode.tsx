import { useEffect, useState } from 'react'
import { useTheme } from 'next-themes'

export function useColorMode() {
  const [mounted, setMounted] = useState(false)
  const { theme, setTheme } = useTheme()

  useEffect(() => setMounted(true), [])
  if (!mounted) return null

  return () => setTheme(theme === 'dark' ? 'light' : 'dark')
}

import create from 'zustand'

export const useColorStore = create((set: any) => ({
  dark: true,
  toggleColorMode: () => set((state: any) => ({ dark: !state.dark }))
}))
