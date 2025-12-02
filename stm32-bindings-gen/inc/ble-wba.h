#ifndef BLE_WBA_BINDINGS_H_
#define BLE_WBA_BINDINGS_H_

#ifdef __cplusplus
extern "C" {
#endif

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include "cmsis_compiler.h"

/* Toolchain abstraction utilities */


/* Link Layer command interface */
#include "bsp.h"
#include "common_types.h"
#include "dtm.h"
#include "event_manager.h"
#include "evnt_schdlr_gnrc_if.h"
#include "hci.h"
#include "ll_error.h"
#include "ll_intf.h"
#include "ll_intf_cmn.h"
#include "mac_host_intf.h"
#include "mem_intf.h"
#include "os_wrapper.h"
#include "platform.h"
#include "power_table.h"
#include "pta.h"
#include "ral.h"
#include "rfd_dev_config.h"


/* Core BLE stack */
#include "blestack.h"
#include "ble_bufsize.h"
#include "ble_codec.h"
#include "ble_const.h"
#include "ble_core.h"
#include "ble_defs.h"
#include "ble_legacy.h"
#include "ble_std.h"
#include "bleplat.h"

/* Auto-generated ACI command definitions */
#include "auto/ble_events.h"
#include "auto/ble_gap_aci.h"
#include "auto/ble_gatt_aci.h"
#include "auto/ble_hal_aci.h"
#include "auto/ble_hci_le.h"
#include "auto/ble_l2cap_aci.h"
#include "auto/ble_raw_api.h"
#include "auto/ble_types.h"
#include "auto/ble_vs_codes.h"

/* BLE Audio stack */
#include "ble_audio_stack.h"
#include "ble_audio_plat.h"
#include "audio_types.h"
#include "bap_bufsize.h"
#include "bap_types.h"
#include "cap.h"
#include "cap_types.h"
#include "ccp.h"
#include "ccp_types.h"
#include "csip.h"
#include "csip_types.h"
#include "ltv_utils.h"
#include "mcp.h"
#include "mcp_types.h"
#include "micp.h"
#include "micp_types.h"
#include "vcp.h"
#include "vcp_types.h"

/* Codec manager interfaces */
#include "codec_if.h"
#include "codec_mngr.h"

/* LC3 codec interfaces */
#include "LC3.h"
#include "LC3_decoder.h"
#include "LC3_encoder.h"

#ifdef __cplusplus
} /* extern "C" */
#endif

#endif /* BLE_WBA_BINDINGS_H_ */
