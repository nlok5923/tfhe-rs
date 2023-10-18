# Benchmarks

Due to their nature, homomorphic operations are naturally slower than their cleartext equivalents. Some timings are exposed for basic operations. For completeness, benchmarks for other libraries are also given.

{% hint style="info" %}
All benchmarks were launched on an AWS m6i.metal with the following specifications: Intel(R) Xeon(R) Platinum 8375C CPU @ 2.90GHz and 512GB of RAM.
{% endhint %}

## Integer

This measures the execution time for some operation sets of tfhe-rs::integer (the unsigned version). Note that the timings for `FheInt` (i.e., the signed integers) are similar.

<table data-full-width="true">
<thead>
    <tr>
    <th>Operation \ Type</th>
    <th width="130" align="center">FheUint8</th>
    <th width="130" align="center">FheUint16</th>
    <th width="130" align="center">FheUint32</th>
    <th width="130" align="center">FheUint64</th>
    <th width="130" align="center">FheUint128</th>
    <th width="130" align="center">FheUint256</th></tr>
</thead>
<tbody>
    <tr>
        <td>Negation<br>(<code>-</code>)</td>
        <td align="center">70.9 ms</td>
        <td align="center">99.3 ms</td>
        <td align="center">129 ms</td>
        <td align="center">180 ms</td>
        <td align="center">239 ms</td>
        <td align="center">333 ms</td>
    </tr>
    <tr>
        <td><p>Add / Sub</p><p>(<code>+</code>,<code>-</code>)</p></td>
        <td align="center">70.5 ms</td>
        <td align="center">100 ms</td>
        <td align="center">132 ms</td>
        <td align="center">186 ms</td>
        <td align="center">249 ms</td>
        <td align="center">334 ms</td>
    </tr>
    <tr>
        <td>Mul<br>(<code>x</code>)</td>
        <td align="center">144 ms</td>
        <td align="center">216 ms</td>
        <td align="center">333 ms</td>
        <td align="center">832 ms</td>
        <td align="center">2.50 s</td>
        <td align="center">8.85 s</td>
    </tr>
    <tr>
        <td>Equal / NotEqual<br>(<code>eq</code>, <code>ne</code>)</td>
        <td align="center">36.1 ms</td>
        <td align="center">36.5 ms</td>
        <td align="center">57.4 ms</td>
        <td align="center">64.2 ms</td>
        <td align="center">67.3 ms</td>
        <td align="center">78.1 ms</td>
    </tr>
    <tr>
        <td>Comparisons<br>(<code>ge</code>, <code>gt</code>, <code>le</code>, <code>lt</code>)</td>
        <td align="center">52.6 ms</td>
        <td align="center">73.1 ms</td>
        <td align="center">98.8 ms</td>
        <td align="center">124 ms</td>
        <td align="center">165 ms</td>
        <td align="center">201 ms</td>
    </tr>
    <tr>
        <td><p>Max / Min </p><p>(<code>max</code>,<code>min</code>)</p></td>
        <td align="center">76.2 ms</td>
        <td align="center">102 ms</td>
        <td align="center">135 ms</td>
        <td align="center">171 ms</td>
        <td align="center">212 ms</td>
        <td align="center">301 ms</td>
    </tr>
    <tr>
        <td><p>Bitwise operations</p><p>(<code>&#x26;</code>, <code>|</code>, <code>^</code>)</p></td>
        <td align="center">19.4 ms</td>
        <td align="center">20.3 ms</td>
        <td align="center">21.0 ms</td>
        <td align="center">27.2 ms</td>
        <td align="center">31.6 ms</td>
        <td align="center">40.2 ms</td>
    </tr>
    <tr>
        <td><p>Div / Rem</p><p>(<code>/</code>, <code>%</code>)</p></td>
        <td align="center">729 ms</td>
        <td align="center">1.93 s</td>
        <td align="center">4.81 s</td>
        <td align="center">12.2 s</td>
        <td align="center">30.7 s</td>
        <td align="center">89.6 s</td>
    </tr>
    <tr>
        <td><p>Left / Right Shifts</p><p>(<code>&#x3C;&#x3C;</code>, <code>>></code>)</p></td>
        <td align="center">99.4 ms</td>
        <td align="center">129 ms</td>
        <td align="center">180 ms</td>
        <td align="center">243 ms</td>
        <td align="center">372 ms</td>
        <td align="center">762 ms</td>
    </tr>
    <tr>
        <td><p>Left / Right Rotations</p><p> (<code>left_rotate</code>, <code>right_rotate</code>)</p></td>
        <td align="center">103 ms</td>
        <td align="center">128 ms</td>
        <td align="center">182 ms</td>
        <td align="center">241 ms</td>
        <td align="center">374 ms</td>
        <td align="center">763 ms</td>
    </tr>
</tbody>
</table>

All timings are related to parallelized Radix-based integer operations, where each block is encrypted using the default parameters (i.e., PARAM\_MESSAGE\_2\_CARRY\_2\_KS\_PBS, more information about parameters can be found [here](../fine\_grained\_api/shortint/parameters.md)). To ensure predictable timings, the operation flavor is the `default` one: the carry is propagated if needed. The operation costs may be reduced by using `unchecked`, `checked`, or `smart`.

## Shortint

This measures the execution time for some operations using various parameter sets of tfhe-rs::shortint. Except for `unchecked_add`, all timings are related to the `default` operations. This flavor ensures predictable timings for an operation along the entire circuit by clearing the carry space after each operation.

This uses the Concrete FFT + AVX-512 configuration.

| Parameter set                      | MESSAGE\_1<br>CARRY\_1 | MESSAGE\_2<br>CARRY\_2 | MESSAGE\_3<br>CARRY\_3 | MESSAGE\_4<br>CARRY\_4 |
|------------------------------------| :--------------------: | :--------------------: | :--------------------: | :--------------------: |
| unchecked\_add                     | 348 ns                 | 413 ns                 | 2.95 µs                | 12.1 µs                |
| add                                | 7.59 ms                | 17.0 ms                | 121 ms                 | 835 ms                 |
| mul\_lsb                           | 8.13 ms                | 16.8 ms                | 121 ms                 | 827 ms                 |
| keyswitch\_programmable\_bootstrap | 7.28 ms                | 16.6  ms               | 121 ms                 | 811 ms                 |


## Boolean

This measures the execution time of a single binary Boolean gate.

### tfhe-rs::boolean.

| Parameter set                                        | Concrete FFT + AVX-512 |
| ---------------------------------------------------- | :--------------------: |
| DEFAULT\_PARAMETERS\_KS\_PBS                         |         9.19 ms        |
| PARAMETERS\_ERROR\_PROB\_2\_POW\_MINUS\_165\_KS\_PBS |         14.1 ms        |
| TFHE\_LIB\_PARAMETERS                                |         10.0 ms        |

### tfhe-lib.

Using the same m6i.metal machine as the one for tfhe-rs, the timings are:

| Parameter set                                    | spqlios-fma |
| ------------------------------------------------ | :---------: |
| default\_128bit\_gate\_bootstrapping\_parameters |   15.4 ms   |

### OpenFHE (v1.1.1).

Following the official instructions from OpenFHE, `clang14` and the following command are used to setup the project: `cmake -DNATIVE_SIZE=32 -DWITH_NATIVEOPT=ON -DCMAKE_C_COMPILER=clang -DCMAKE_CXX_COMPILER=clang++ -DWITH_OPENMP=OFF ..`

To use the HEXL library, the configuration used is as follows:

```bash
export CXX=clang++
export CC=clang

scripts/configure.sh
Release -> y
hexl -> y

scripts/build-openfhe-development-hexl.sh
```

Using the same m6i.metal machine as the one for tfhe-rs, the timings are:

| Parameter set                     |   GINX  | GINX w/ Intel HEXL |
| --------------------------------- | :-----: | :----------------: |
| FHEW\_BINGATE/STD128\_OR          | 40.2 ms |       31.0 ms      |
| FHEW\_BINGATE/STD128\_LMKCDEY\_OR | 38.6 ms |       28.4 ms      |

## How to reproduce TFHE-rs benchmarks

TFHE-rs benchmarks can be easily reproduced from [source](https://github.com/zama-ai/tfhe-rs).

```shell
#Boolean benchmarks:
make AVX512_SUPPORT=ON bench_boolean

#Integer benchmarks:
make AVX512_SUPPORT=ON bench_integer

#Shortint benchmarks:
make AVX512_SUPPORT=ON bench_shortint
```

If the host machine does not support AVX512, then turning on `AVX512_SUPPORT` will not provide any speed-up.
