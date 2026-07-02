using MDreader.Util;
using Xunit;

namespace MDreader.Tests;

public class ContentHashTests
{
    [Fact]
    public void EmptyString() =>
        Assert.Equal(
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            ContentHash.Sha256Hex(""));

    [Fact]
    public void KnownVectorAbc() =>
        Assert.Equal(
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            ContentHash.Sha256Hex("abc"));

    [Fact]
    public void StableForSameInput() =>
        Assert.Equal(ContentHash.Sha256Hex("hello"), ContentHash.Sha256Hex("hello"));

    [Fact]
    public void DifferentForDifferentInput() =>
        Assert.NotEqual(ContentHash.Sha256Hex("a"), ContentHash.Sha256Hex("b"));

    [Fact]
    public void OutputIs64LowerHexChars()
    {
        var hex = ContentHash.Sha256Hex("some markdown content");
        Assert.Equal(64, hex.Length);
        Assert.All(hex, c => Assert.Contains(c, "0123456789abcdef"));
    }
}
