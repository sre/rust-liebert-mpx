# Liebert MPX PDU interface

This Rust crate can be used to access information from Liebert MPX PDUs
(power distribution units) by using its web interface.

## Tested Hardware

 * PE Modules: MPXPEM-EHAXXR30
 * BR Modules: MPXBRM-ERBC6N1N, MPXBRM-ERBC6N2N, MPXBRM-ERBC6N3N

## Supported Features

 * read interface
   * getting a list of all receptacles
   * getting a list of all events/alarms
   * getting detailed information about the PDU's power input module(s) (PEM)
   * getting detailed information about the PDU's branch module(s) (BRM)
   * getting detailed information about the PDU's receptacle(s)
 * write interface
   * sending test event
   * clearing PDU/Branch/Receptacle accumulated energy
   * setting receptacles power state
   * identifing receptacles
   * PDU/Branch/Receptacle settings
