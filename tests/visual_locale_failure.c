// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#define _GNU_SOURCE
#include <dlfcn.h>
#include <errno.h>
#include <locale.h>
#include <stdlib.h>
#include <unistd.h>
static unsigned calls;
static locale_t live;
locale_t newlocale(int mask, const char *name, locale_t base)
{
  if (++calls == 2) { errno=ENOMEM; return (locale_t)0; }
  if (live) _exit(91);
  locale_t (*next)(int,const char *,locale_t)=dlsym(RTLD_NEXT,"newlocale");
  if (!next) _exit(92);
  live=next(mask,name,base);
  return live;
}
void freelocale(locale_t locale)
{
  if (!locale || locale!=live) _exit(93);
  void (*next)(locale_t)=dlsym(RTLD_NEXT,"freelocale");
  if (!next) _exit(94);
  next(locale); live=(locale_t)0;
}
__attribute__((destructor)) static void check_released(void)
{
  if (live) _exit(95);
}
