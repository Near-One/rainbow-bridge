
      import { storage, near, base64 } from "./near";
      import { JSONEncoder } from "./json/encoder";
      import { JSONDecoder, ThrowingJSONHandler, DecoderState } from "./json/decoder";
      import {sayHi as wrapped_sayHi, whoSaidHi as wrapped_whoSaidHi} from "./main";
      
      // Runtime functions
      @external("env", "return_value")
      declare function return_value(value_len: usize, value_ptr: usize): void;
    
import {context as context,storage as storage,near as near} from "./near";
export class __near_ArgsParser_sayHi extends ThrowingJSONHandler {
        buffer: Uint8Array;
        decoder: JSONDecoder<__near_ArgsParser_sayHi>;
        handledRoot: boolean = false;
      
setNull(name: string): void {

      super.setNull(name);
    }

      pushObject(name: string): bool {
if (!this.handledRoot) {
      assert(name == null);
      this.handledRoot = true;
      return true;
    } else {
      assert(name != null);
    }

        return super.pushObject(name);
      }

      pushArray(name: string): bool {

        return super.pushArray(name);
      }
}
export function sayHi(): void {
      // Reading input bytes.
      let json = storage._internalReadBytes(4, 0, 0);
      let handler = new __near_ArgsParser_sayHi();
      handler.buffer = json;
      handler.decoder = new JSONDecoder<__near_ArgsParser_sayHi>(handler);
      handler.decoder.deserialize(json);
wrapped_sayHi(

);
}
export class __near_ArgsParser_whoSaidHi extends ThrowingJSONHandler {
        buffer: Uint8Array;
        decoder: JSONDecoder<__near_ArgsParser_whoSaidHi>;
        handledRoot: boolean = false;
      
setNull(name: string): void {

      super.setNull(name);
    }

      pushObject(name: string): bool {
if (!this.handledRoot) {
      assert(name == null);
      this.handledRoot = true;
      return true;
    } else {
      assert(name != null);
    }

        return super.pushObject(name);
      }

      pushArray(name: string): bool {

        return super.pushArray(name);
      }
}
export function whoSaidHi(): void {
      // Reading input bytes.
      let json = storage._internalReadBytes(4, 0, 0);
      let handler = new __near_ArgsParser_whoSaidHi();
      handler.buffer = json;
      handler.decoder = new JSONDecoder<__near_ArgsParser_whoSaidHi>(handler);
      handler.decoder.deserialize(json);
let result = wrapped_whoSaidHi(

);

        let encoder = new JSONEncoder();
      
if (result != null) {
            encoder.setString(null, result);
          } else {
            encoder.setNull(null);
          }

        let val = encoder.serialize();
        return_value(val.byteLength, <usize>val.buffer);
      
}