// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#define _GNU_SOURCE
#include <locale.h>
#include <errno.h>
#include <stdlib.h>

// Fault injection is confined to a separate Rust test process.
locale_t newlocale(int mask, const char *name, locale_t base)
{
  (void)mask; (void)name; (void)base;
  errno = ENOMEM;
  return (locale_t)0;
}
void freelocale(locale_t locale)
{
  (void)locale;
  // A failed newlocale must not be passed to cleanup.
  _Exit(91);
}
