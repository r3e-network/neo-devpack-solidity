using FluentAssertions;
using Neo.Sol.Runtime.Events;
using NUnit.Framework;
using UInt160 = Neo.UInt160;

namespace Neo.Sol.Runtime.Tests.Events;

[TestFixture]
public class EventManagerTests
{
    private static UInt160 SampleAddress(byte fill)
        => new(new byte[]
        {
            fill, fill, fill, fill, fill, fill, fill, fill, fill, fill,
            fill, fill, fill, fill, fill, fill, fill, fill, fill, fill,
        });

    [Test]
    public void Constructor_StartsWithZeroEventCount()
    {
        var manager = new EventManager(SampleAddress(0x11));

        manager.Should().NotBeNull();
        manager.GetEventCount().Should().Be(0);
    }

    [Test]
    public void EventLog_ToNotificationObject_RoundTrips()
    {
        var original = new EventLog
        {
            Address = SampleAddress(0x22),
            Topics = new[]
            {
                new byte[32],
                Enumerable.Repeat((byte)0xAB, 32).ToArray(),
            },
            Data = new byte[] { 0x01, 0x02, 0x03, 0x04 },
            EventSignature = "Transfer(address,address,uint256)",
        };

        var notification = original.ToNotificationObject();
        var restored = EventLog.FromNotificationObject(notification);

        restored.Address.Should().Be(original.Address);
        restored.Topics.Should().BeEquivalentTo(original.Topics);
        restored.Data.Should().BeEquivalentTo(original.Data);
        restored.EventSignature.Should().Be(original.EventSignature);
    }

    [Test]
    public void EventFilter_MatchesAddressAndTopics()
    {
        var topic0 = Enumerable.Repeat((byte)0x10, 32).ToArray();
        var topic1 = Enumerable.Repeat((byte)0x20, 32).ToArray();
        var log = new EventLog
        {
            Address = SampleAddress(0x33),
            Topics = new[] { topic0, topic1 },
            Data = new byte[] { 0x99 },
            EventSignature = "Example(uint256)",
        };

        var filter = new EventFilter
        {
            Address = SampleAddress(0x33),
            Topic0 = topic0,
            Topic1 = topic1,
        };

        filter.Matches(log).Should().BeTrue();
    }

    [Test]
    public void EventFilter_RejectsMismatchedTopic()
    {
        var log = new EventLog
        {
            Address = SampleAddress(0x44),
            Topics = new[]
            {
                Enumerable.Repeat((byte)0xAA, 32).ToArray(),
            },
            Data = Array.Empty<byte>(),
            EventSignature = "Paused(address)",
        };

        var filter = new EventFilter
        {
            Address = SampleAddress(0x44),
            Topic0 = Enumerable.Repeat((byte)0xBB, 32).ToArray(),
        };

        filter.Matches(log).Should().BeFalse();
    }
}
