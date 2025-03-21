# FTNet: FifthTry Network

FifthTry Network, FTN, is an open source, general purpose, sovereign, 
decentralized (no blockchain/crypto stuff), peer to peer social network, free 
from any corporate control. Data stays with the user, and devices controlled by 
the user, and not with some central company.

FTN is available as an binary that you can download and run on your computer. We
support Linux, Windows and MacOS from day one. We also want to create Apps that
can be distributed through App Stores, and also support mobile devices.

## FTNet: The Sovereign Network

FifthTry Network is a sovereign network. This means that the user is in control
of their identity, there is no IP address that has to be leased/bought from
central authorities, no account to create on any central website, there is no 
DNS service that can be taken down, etc.

Unlike other P2P networks, that are designed for single purpose, eg BitTorrent 
for file sharing, BitCoin for crypto/finance, FTN is a general purpose network, 
and designed for easy app development and extension, this is done by using 
[fastn][fastn] as the application development framework.

Technical Details: FTN is built on top of [iroh][iroh], and uses [BitTorrent's 
Mainline DHT][MainlineDHT] for peer discovery.

[fastn]: https://fastn.com
[iroh]: https://www.iroh.computer
[MainlineDHT]: https://en.wikipedia.org/wiki/Mainline_DHT

## FTN Identities

Once you install FTN, the first thing you do is create an identity.

The central concept of FifthTry Network is identities. An identity is an alias
for a person or a group of people (e.g., a company, a school, etc.).

The identity is a public key, and the private key is used to sign or encrypt
data and messages sent by that identity. The public key has a compact 
representation and can be shared easily over Email, WhatsApp, QR code, etc.

Once you create an identity, you share the public key with your friends and
family, they add you to their network (after creating their own identity), and
then you can share messages, files, etc., with them.

You can create multiple identities, and each identity is independent of each
other; this facilitates anonymity, for example.

## FTN Devices

An identity can own one or more devices. Each device is also identified by its 
public key. Example of a device could be a folder containing some files, meaning
an identity can have a folder, and they want to share that folder with their
social network, so they create a device for that folder.

Another example could be a HTTP or TCP (UDP etc) service, say if the identity
has a local web service running, or a a VNC server running, they can create a
device for that server, and share that device with their social network.

Other examples of devices could be a Printer, a USB device, a Bluetooth device,
a webcam, and external hard drive. 

For creating a "device", FTN has to be installed on the physical machine, and
the primary identity has to be configured (either identity was created on that
physical machine, or on another machine). Any service (USB, Bluetooth, HTTP)
accessible from that machine can be shared as a device.

## `fastn` Devices

Any fastn package can be installed on any FTN identity (FTN identity is also
a device of kind `identity`, which is a HTTP service powered by `fastn`, and 
stored in `~/.ftn/<identity>`). 

When creating a new device of `fastn` kind, the user can point the device to the 
folder in which fastn package is installed. 

The `fastn` package for `identity` is special in that it must have a `fastn app`
called `lets-auth` installed, which is the authentication service for `fastn`,
and this is responsible for configuring which identities and devices can be
accessed by which other identities and devices.

## Licence

FTNet is licensed under the [UPL](LICENSE) license. UPL is MIT like license, 
with Apache 2.0 patent grant clause.

## Contributing

We welcome contributions to FTNet. Please read the [CONTRIBUTING.md][cont]
file for details on how to contribute.

[cont]: CONTRIBUTING.md
