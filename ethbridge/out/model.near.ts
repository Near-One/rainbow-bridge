
      import { storage, near, base64 } from "./near";
      import { JSONEncoder } from "./json/encoder";
      import { JSONDecoder, ThrowingJSONHandler, DecoderState } from "./json/decoder";
      import {} from "./model";
      
      // Runtime functions
      @external("env", "return_value")
      declare function return_value(value_len: usize, value_ptr: usize): void;
    
export function __near_encode_Greeter(
          value: Greeter,
          encoder: JSONEncoder): void {
if (value.text != null) {
            encoder.setString("text", value.text);
          } else {
            encoder.setNull("text");
          }
}
export class __near_JSONHandler_Greeter extends ThrowingJSONHandler {
      buffer: Uint8Array;
      decoder: JSONDecoder<__near_JSONHandler_Greeter>;
      handledRoot: boolean = false;
      value: Greeter;
      
      constructor(value_: Greeter) {
        super();
        this.value = value_;
      }
    
setString(name: string, value: String): void {
if (name == "text") {
            this.value.text = <String>value;
            return;
          }

        super.setString(name, value);
      }
setNull(name: string): void {
if (name == "text") {
        this.value.text = <String>null;
        return;
      }

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

export function __near_decode_Greeter(
        buffer: Uint8Array, state: DecoderState, value: Greeter = null):Greeter {
      if (value == null) {
        value = new Greeter();
      }
      let handler = new __near_JSONHandler_Greeter(value);
      handler.buffer = buffer;
      handler.decoder = new JSONDecoder<__near_JSONHandler_Greeter>(handler);
      handler.decoder.deserialize(buffer, state);
      return value;
    }

export class Greeter {
  text: string;

  constructor(text: string) {
    this.text = text;
  }

  greet(userId: string): string {
    return "Hello, " + userId;
  }


        static decode(json: Uint8Array): Greeter {
          let value = new Greeter();
          value.decode(json);
          return value;
        }

        decode(json: Uint8Array): Greeter {
          __near_decode_Greeter(json, null, this);
          return this;
        }

        private _encoder(): JSONEncoder {
          let encoder: JSONEncoder = new JSONEncoder();
          encoder.pushObject(null);
          __near_encode_Greeter(this, encoder);
          encoder.popObject();
          return encoder;
        }

        encode(): Uint8Array {
          return this._encoder().serialize();
        }

        toString(): string {
          return this._encoder().toString();
        }
      
}