cat samples/jfk.wav.json
{
	"systeminfo": "AVX = 1 | AVX2 = 1 | AVX512 = 0 | FMA = 1 | NEON = 0 | ARM_FMA = 0 | METAL = 0 | F16C = 1 | FP16_VA = 0 | WASM_SIMD = 0 | BLAS = 0 | SSE3 = 1 | SSSE3 = 1 | VSX = 0 | COREML = 0 | OPENVINO = 0 | ",
	"model": {
		"type": "base",
		"multilingual": false,
		"vocab": 51864,
		"audio": {
			"ctx": 1500,
			"state": 512,
			"head": 8,
			"layer": 6
		},
		"text": {
			"ctx": 448,
			"state": 512,
			"head": 8,
			"layer": 6
		},
		"mels": 80,
		"ftype": 1
	},
	"params": {
		"model": "models/ggml-base.en.bin",
		"language": "en",
		"translate": false
	},
	"result": {
		"language": "en"
	},
	"transcription": [
		{
			"timestamps": {
				"from": "00:00:00,000",
				"to": "00:00:11,000"
			},
			"offsets": {
				"from": 0,
				"to": 11000
			},
			"text": " And so my fellow Americans, ask not what your country can do for you, ask what you can do for your country."
		}
	]
}

./main -otxt -m models/ggml-base.en.bin -f samples/jfk.wav
output_txt: saving output to 'samples/jfk.wav.txt'