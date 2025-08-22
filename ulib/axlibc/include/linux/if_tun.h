#ifndef _UAPI__IF_TUN_H
#define _UAPI__IF_TUN_H

/* Features for GSO (TUNSETOFFLOAD). */
#define TUN_F_CSUM    0x01 /* You can hand me unchecksummed packets. */
#define TUN_F_TSO4    0x02 /* I can handle TSO for IPv4 packets */
#define TUN_F_TSO6    0x04 /* I can handle TSO for IPv6 packets */
#define TUN_F_TSO_ECN 0x08 /* I can handle TSO with ECN bits. */
#define TUN_F_UFO     0x10 /* I can handle UFO packets */
#define TUN_F_USO4    0x20 /* I can handle USO for IPv4 packets */
#define TUN_F_USO6    0x40 /* I can handle USO for IPv6 packets */

/* TUNSETIFF ifr flags */
#define IFF_TUN        0x0001
#define IFF_TAP        0x0002
#define IFF_NAPI       0x0010
#define IFF_NAPI_FRAGS 0x0020
/* Used in TUNSETIFF to bring up tun/tap without carrier */
#define IFF_NO_CARRIER 0x0040
#define IFF_NO_PI      0x1000
/* This flag has no real effect */
#define IFF_ONE_QUEUE    0x2000
#define IFF_VNET_HDR     0x4000
#define IFF_TUN_EXCL     0x8000
#define IFF_MULTI_QUEUE  0x0100
#define IFF_ATTACH_QUEUE 0x0200
#define IFF_DETACH_QUEUE 0x0400
/* read-only flag */
#define IFF_PERSIST  0x0800
#define IFF_NOFILTER 0x1000

/* Some IOCTL flags */
// To avoid including some complex Linux headers, we define these IOCTL flags here directly.
#define TUNSETIFF       0x400454ca
#define TUNSETVNETHDRSZ 0x400454d8
#define TUNSETOFFLOAD   0x400454d0

#endif /* _UAPI__IF_TUN_H */
