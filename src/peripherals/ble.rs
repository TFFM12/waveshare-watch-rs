//! BLE advertising control via raw HCI commands.
//!
//! esp-radio's `BleConnector` is a low-level HCI transport — it doesn't
//! provide a GATT server or high-level advertising API. For the smartwatch
//! we only need the device to be discoverable (advertising its name), so
//! we send 3 HCI commands directly:
//!
//!   1. LE Set Advertising Parameters (slow interval = power-friendly)
//!   2. LE Set Advertising Data (Flags + Complete Local Name)
//!   3. LE Set Advertising Enable (on / off)
//!
//! The VHCI interface in ESP-IDF expects H4 transport framing, so every
//! command is prefixed with 0x01 (HCI Command Packet).

use embedded_io::Write;

/// Start BLE advertising as "Rust Watch".
/// Sends HCI commands synchronously via the BleConnector's Write impl.
pub fn start_advertising<W: Write>(hci: &mut W) -> Result<(), W::Error> {
    // 1) LE Set Advertising Parameters
    //    Opcode 0x2006, 15 bytes of params
    //    Interval: 0x0800 (1.28s) — slow to save power
    //    Type: ADV_IND (connectable, undirected)
    //    Channels: all 3 (37, 38, 39)
    hci.write_all(&[
        0x01,                   // H4: HCI command
        0x06, 0x20,             // opcode: LE Set Advertising Parameters
        15,                     // param length
        0x00, 0x08,             // interval min: 0x0800 (1280 * 0.625ms = 800ms)
        0x00, 0x08,             // interval max: 0x0800
        0x00,                   // type: ADV_IND
        0x00,                   // own addr type: public
        0x00,                   // peer addr type
        0, 0, 0, 0, 0, 0,      // peer addr (unused)
        0x07,                   // channel map: all
        0x00,                   // filter policy: any
    ])?;

    // 2) LE Set Advertising Data
    //    Opcode 0x2008, always 32 bytes of param (1 len + 31 data)
    let name = b"Rust Watch";
    let flags_len: u8 = 3;      // AD: [len=2, type=0x01 Flags, val=0x06]
    let name_ad_len: u8 = 1 + name.len() as u8; // [type + name bytes]
    let sig_octets = flags_len + 1 + name_ad_len; // total significant

    let mut cmd = [0u8; 36]; // 1 (H4) + 2 (opcode) + 1 (plen) + 32 (data) = 36
    cmd[0] = 0x01;                          // H4
    cmd[1] = 0x08; cmd[2] = 0x20;          // opcode: LE Set Advertising Data
    cmd[3] = 32;                            // param length (always 32)
    cmd[4] = sig_octets;                    // significant octets count
    // Flags AD structure
    cmd[5] = 2;                             // length of this AD
    cmd[6] = 0x01;                          // AD type: Flags
    cmd[7] = 0x06;                          // General Discoverable + BR/EDR Not Supported
    // Complete Local Name AD structure
    cmd[8] = name_ad_len;
    cmd[9] = 0x09;                          // AD type: Complete Local Name
    cmd[10..10 + name.len()].copy_from_slice(name);
    // Remaining bytes are zero (padding)
    hci.write_all(&cmd)?;

    // 3) LE Set Advertising Enable
    //    Opcode 0x200A, 1 byte param = 0x01 (enable)
    hci.write_all(&[
        0x01,           // H4
        0x0A, 0x20,     // opcode: LE Set Advertising Enable
        1,              // param length
        0x01,           // enable
    ])?;

    Ok(())
}

/// Stop BLE advertising.
pub fn stop_advertising<W: Write>(hci: &mut W) -> Result<(), W::Error> {
    hci.write_all(&[
        0x01,           // H4
        0x0A, 0x20,     // opcode: LE Set Advertising Enable
        1,              // param length
        0x00,           // disable
    ])?;
    Ok(())
}

/// Parse a BLE MAC address from "AA:BB:CC:DD:EE:FF" text.
/// Returns little-endian bytes as expected by HCI commands.
pub fn parse_peer_addr(s: &str) -> Option<[u8; 6]> {
    let mut out = [0u8; 6];
    let mut idx = 0usize;
    let bytes = s.as_bytes();
    let mut i = 0usize;

    while i + 1 < bytes.len() && idx < 6 {
        let hi = hex_nibble(bytes[i])?;
        let lo = hex_nibble(bytes[i + 1])?;
        // HCI expects peer address in little-endian order.
        out[5 - idx] = (hi << 4) | lo;
        idx += 1;
        i += 2;
        if idx < 6 {
            if i >= bytes.len() || bytes[i] != b':' {
                return None;
            }
            i += 1;
        }
    }

    if idx == 6 && i == bytes.len() {
        Some(out)
    } else {
        None
    }
}

/// Request a BLE LE connection to a specific peer.
/// This sends HCI LE Create Connection (0x200D).
pub fn connect_peer<W: Write>(
    hci: &mut W,
    peer_addr_le: [u8; 6],
    peer_is_random_addr: bool,
) -> Result<(), W::Error> {
    let peer_addr_type = if peer_is_random_addr { 0x01 } else { 0x00 };
    hci.write_all(&[
        0x01,       // H4: command packet
        0x0D, 0x20, // LE Create Connection
        25,         // param length
        0x10, 0x00, // scan interval (16 * 0.625ms = 10ms)
        0x10, 0x00, // scan window
        0x00,       // initiator filter policy: use peer address below
        peer_addr_type,
        peer_addr_le[0],
        peer_addr_le[1],
        peer_addr_le[2],
        peer_addr_le[3],
        peer_addr_le[4],
        peer_addr_le[5],
        0x00,       // own address type: public
        0x18, 0x00, // conn interval min (30ms)
        0x28, 0x00, // conn interval max (50ms)
        0x00, 0x00, // conn latency
        0xC8, 0x00, // supervision timeout (2s)
        0x00, 0x00, // min CE length
        0x00, 0x00, // max CE length
    ])?;
    Ok(())
}

/// Cancel an ongoing BLE LE connection procedure.
pub fn cancel_connect<W: Write>(hci: &mut W) -> Result<(), W::Error> {
    hci.write_all(&[
        0x01,
        0x0E, 0x20, // LE Create Connection Cancel
        0,
    ])?;
    Ok(())
}

fn hex_nibble(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}
