using System.Numerics;
using System.Security.Cryptography;
using Org.BouncyCastle.Crypto.Digests;
namespace Neo.Sol.Runtime.Crypto;

/// <summary>
/// Cryptographic library providing EVM-compatible hash functions and signature verification
/// </summary>
public static class CryptoLib
{
    /// <summary>
    /// Compute Keccak256 hash (used throughout EVM)
    /// </summary>
    /// <param name="data">Data to hash</param>
    /// <returns>32-byte hash</returns>
    public static byte[] Keccak256(byte[] data)
    {
        var digest = new KeccakDigest(256);
        digest.BlockUpdate(data, 0, data.Length);
        var result = new byte[digest.GetDigestSize()];
        digest.DoFinal(result, 0);
        return result;
    }
    
    /// <summary>
    /// Compute Keccak256 hash from multiple byte arrays
    /// </summary>
    /// <param name="parts">Data parts to concatenate and hash</param>
    /// <returns>32-byte hash</returns>
    public static byte[] Keccak256(params byte[][] parts)
    {
        var digest = new KeccakDigest(256);
        foreach (var part in parts)
        {
            digest.BlockUpdate(part, 0, part.Length);
        }
        var result = new byte[digest.GetDigestSize()];
        digest.DoFinal(result, 0);
        return result;
    }
    
    /// <summary>
    /// Compute SHA256 hash
    /// </summary>
    /// <param name="data">Data to hash</param>
    /// <returns>32-byte hash</returns>
    public static byte[] Sha256(byte[] data)
    {
        using var sha256 = SHA256.Create();
        return sha256.ComputeHash(data);
    }
    
    /// <summary>
    /// Compute SHA256 hash from multiple byte arrays
    /// </summary>
    /// <param name="parts">Data parts to concatenate and hash</param>
    /// <returns>32-byte hash</returns>
    public static byte[] Sha256(params byte[][] parts)
    {
        using var sha256 = SHA256.Create();
        var combined = CombineArrays(parts);
        return sha256.ComputeHash(combined);
    }
    
    /// <summary>
    /// Recover public key from signature and message hash (ecrecover)
    /// </summary>
    /// <param name="messageHash">32-byte message hash</param>
    /// <param name="signature">64-byte signature (r + s)</param>
    /// <param name="recoveryId">Recovery ID (0 or 1)</param>
    /// <returns>Recovered public key (64 bytes uncompressed, without 0x04 prefix)</returns>
    public static byte[]? EcRecover(byte[] messageHash, byte[] signature, int recoveryId)
    {
        if (messageHash.Length != 32)
            throw new ArgumentException("Message hash must be 32 bytes");
        if (signature.Length != 64)
            throw new ArgumentException("Signature must be 64 bytes");
        if (recoveryId < 0 || recoveryId > 3)
            throw new ArgumentException("Recovery ID must be 0, 1, 2, or 3");
            
        try
        {
            // Extract r and s from signature
            var r = new byte[32];
            var s = new byte[32];
            Array.Copy(signature, 0, r, 0, 32);
            Array.Copy(signature, 32, s, 0, 32);
            
            var rBig = new BigInteger(r, isUnsigned: true, isBigEndian: true);
            var sBig = new BigInteger(s, isUnsigned: true, isBigEndian: true);
            
            // Secp256k1 curve parameters
            var p = BigInteger.Parse("115792089237316195423570985008687907853269984665640564039457584007913129639935");
            var n = BigInteger.Parse("115792089237316195423570985008687907852837564279074904382605163141518161494337");
            var a = BigInteger.Zero;
            var b = new BigInteger(7);
            var gx = BigInteger.Parse("55066263022277343669578718895168534326250603453777594175500187360389116729240");
            var gy = BigInteger.Parse("32670510020758816978083085130507043184471273380659243275938904335757337482424");
            
            // Validate r and s
            if (rBig <= 0 || rBig >= n || sBig <= 0 || sBig >= n)
                return null;
                
            // Calculate recovery point
            var x = rBig + (recoveryId >= 2 ? n : 0);
            if (x >= p) return null;
            
            // Calculate y coordinate
            var alpha = (BigInteger.ModPow(x, 3, p) + a * x + b) % p;
            var beta = ModularSqrt(alpha, p);
            if (beta == null) return null;
            
            var y = (beta % 2) != (recoveryId % 2) ? p - beta : beta;
            
            // Create point R
            var R = new ECPoint(x, y.Value);
            
            // Calculate recovery
            var e = new BigInteger(messageHash, isUnsigned: true, isBigEndian: true);
            var rInv = ModularInverse(rBig, n);
            
            if (rInv == null) return null;
            
            // Q = r^(-1) * (sR - eG)
            var sR = ScalarMultiply(R, sBig, p);
            var eG = ScalarMultiply(new ECPoint(gx, gy), e, p);
            var sR_minus_eG = PointSubtract(sR, eG, p);
            var Q = ScalarMultiply(sR_minus_eG, rInv.Value, p);
            
            // Return uncompressed public key (without 0x04 prefix)
            var pubKey = new byte[64];
            var qxBytes = Q.X.ToByteArray(isUnsigned: true, isBigEndian: true);
            var qyBytes = Q.Y.ToByteArray(isUnsigned: true, isBigEndian: true);
            
            // Pad to 32 bytes
            Array.Copy(qxBytes, 0, pubKey, 32 - qxBytes.Length, qxBytes.Length);
            Array.Copy(qyBytes, 0, pubKey, 64 - qyBytes.Length, qyBytes.Length);
            
            return pubKey;
        }
        catch
        {
            return null;
        }
    }
    
    /// <summary>
    /// Compute Ethereum-style address from public key
    /// </summary>
    /// <param name="publicKey">64-byte uncompressed public key</param>
    /// <returns>20-byte address</returns>
    public static byte[] PublicKeyToAddress(byte[] publicKey)
    {
        if (publicKey.Length != 64)
            throw new ArgumentException("Public key must be 64 bytes");
            
        var hash = Keccak256(publicKey);
        var address = new byte[20];
        Array.Copy(hash, 12, address, 0, 20);
        return address;
    }
    
    /// <summary>
    /// Verify ECDSA signature
    /// </summary>
    /// <param name="messageHash">32-byte message hash</param>
    /// <param name="signature">64-byte signature</param>
    /// <param name="publicKey">64-byte public key</param>
    /// <returns>True if signature is valid</returns>
    public static bool VerifySignature(byte[] messageHash, byte[] signature, byte[] publicKey)
    {
        if (messageHash.Length != 32 || signature.Length != 64 || publicKey.Length != 64)
            return false;
            
        try
        {
            var x = new byte[32];
            var y = new byte[32];
            Array.Copy(publicKey, 0, x, 0, 32);
            Array.Copy(publicKey, 32, y, 0, 32);

            var parameters = new ECParameters
            {
                Curve = ECCurve.CreateFromValue("1.3.132.0.10"),
                Q = new System.Security.Cryptography.ECPoint
                {
                    X = x,
                    Y = y
                }
            };

            using var ecdsa = ECDsa.Create(parameters);
            return ecdsa.VerifyHash(
                messageHash,
                signature,
                DSASignatureFormat.IeeeP1363FixedFieldConcatenation
            );
        }
        catch
        {
            return false;
        }
    }
    
    // Helper methods for elliptic curve operations
    
    private static BigInteger? ModularSqrt(BigInteger a, BigInteger p)
    {
        if (BigInteger.ModPow(a, (p - 1) / 2, p) != 1)
            return null;
            
        if (p % 4 == 3)
        {
            return BigInteger.ModPow(a, (p + 1) / 4, p);
        }
        
        // Tonelli-Shanks algorithm for general case
        var s = 0;
        var q = p - 1;
        while (q % 2 == 0)
        {
            q /= 2;
            s++;
        }
        
        if (s == 1)
        {
            return BigInteger.ModPow(a, (p + 1) / 4, p);
        }
        
        // Find a quadratic non-residue
        var z = new BigInteger(2);
        while (BigInteger.ModPow(z, (p - 1) / 2, p) != p - 1)
        {
            z++;
        }
        
        var c = BigInteger.ModPow(z, q, p);
        var r = BigInteger.ModPow(a, (q + 1) / 2, p);
        var t = BigInteger.ModPow(a, q, p);
        var m = s;
        
        while (t != 1)
        {
            var i = 1;
            var temp = (t * t) % p;
            while (temp != 1 && i < m)
            {
                temp = (temp * temp) % p;
                i++;
            }
            
            var b = BigInteger.ModPow(c, BigInteger.Pow(2, m - i - 1), p);
            r = (r * b) % p;
            c = (b * b) % p;
            t = (t * c) % p;
            m = i;
        }
        
        return r;
    }
    
    private static BigInteger? ModularInverse(BigInteger a, BigInteger m)
    {
        if (BigInteger.GreatestCommonDivisor(a, m) != 1)
            return null;
            
        // Extended Euclidean algorithm
        var (_, x, _) = ExtendedGcd(a, m);
        return ((x % m) + m) % m;
    }
    
    private static (BigInteger gcd, BigInteger x, BigInteger y) ExtendedGcd(BigInteger a, BigInteger b)
    {
        if (b == 0)
            return (a, 1, 0);
            
        var (gcd, x1, y1) = ExtendedGcd(b, a % b);
        var x = y1;
        var y = x1 - (a / b) * y1;
        return (gcd, x, y);
    }
    
    private static ECPoint ScalarMultiply(ECPoint point, BigInteger scalar, BigInteger p)
    {
        if (scalar == 0)
            return new ECPoint(BigInteger.Zero, BigInteger.Zero, true); // Point at infinity
            
        if (scalar == 1)
            return point;
            
        if (scalar % 2 == 0)
        {
            var half = ScalarMultiply(point, scalar / 2, p);
            return PointDouble(half, p);
        }
        
        return PointAdd(point, ScalarMultiply(point, scalar - 1, p), p);
    }
    
    private static ECPoint PointAdd(ECPoint p1, ECPoint p2, BigInteger fieldP)
    {
        if (p1.IsInfinity) return p2;
        if (p2.IsInfinity) return p1;
        
        if (p1.X == p2.X)
        {
            if (p1.Y == p2.Y)
            {
                return PointDouble(p1, fieldP);
            }
            else
            {
                return new ECPoint(BigInteger.Zero, BigInteger.Zero, true); // Point at infinity
            }
        }
        
        var deltaX = (p2.X - p1.X + fieldP) % fieldP;
        var deltaY = (p2.Y - p1.Y + fieldP) % fieldP;
        var deltaXInv = ModularInverse(deltaX, fieldP);
        
        if (deltaXInv == null)
            return new ECPoint(BigInteger.Zero, BigInteger.Zero, true);
            
        var slope = (deltaY * deltaXInv.Value) % fieldP;
        var x3 = (slope * slope - p1.X - p2.X + 2 * fieldP) % fieldP;
        var y3 = (slope * (p1.X - x3) - p1.Y + fieldP) % fieldP;
        
        return new ECPoint(x3, y3);
    }
    
    private static ECPoint PointDouble(ECPoint point, BigInteger fieldP)
    {
        if (point.IsInfinity || point.Y == 0)
            return new ECPoint(BigInteger.Zero, BigInteger.Zero, true);
            
        var twoY = (2 * point.Y) % fieldP;
        var twoYInv = ModularInverse(twoY, fieldP);
        
        if (twoYInv == null)
            return new ECPoint(BigInteger.Zero, BigInteger.Zero, true);
            
        var slope = (3 * point.X * point.X * twoYInv.Value) % fieldP;
        var x3 = (slope * slope - 2 * point.X + fieldP) % fieldP;
        var y3 = (slope * (point.X - x3) - point.Y + fieldP) % fieldP;
        
        return new ECPoint(x3, y3);
    }
    
    private static ECPoint PointSubtract(ECPoint p1, ECPoint p2, BigInteger fieldP)
    {
        var negP2 = new ECPoint(p2.X, (fieldP - p2.Y) % fieldP);
        return PointAdd(p1, negP2, fieldP);
    }
    
    private static byte[] CombineArrays(byte[][] arrays)
    {
        var totalLength = arrays.Sum(arr => arr.Length);
        var result = new byte[totalLength];
        var offset = 0;
        
        foreach (var array in arrays)
        {
            Array.Copy(array, 0, result, offset, array.Length);
            offset += array.Length;
        }
        
        return result;
    }
}

/// <summary>
/// Represents a point on an elliptic curve
/// </summary>
internal record ECPoint(BigInteger X, BigInteger Y, bool IsInfinity = false);
