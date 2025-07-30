a = [(2242, 38.902935), (2306, 37.702557), (2340, 37.38681), (3024, 28.709398), (3470, 25.00569), (2764, 31.536888), (3368, 25.79274), (2356, 36.909767), (2124, 41.235416), (2144, 40.75212), (2276, 38.370335), (2472, 35.040752), (2988, 28.757198), (3420, 25.395922), (2996, 28.863111)]

# have: samples/frame and frames/sec
# want: samples/sec

# samples/sec = (samples/frame) * (frames/sec)

rates = []

for thing in a:
    samples_per_frame = thing[0] // 2
    fps = thing[1]

    sample_rate = samples_per_frame * fps

    rates.append(sample_rate)

    print(f"Sample rate: {sample_rate}")

print(f"Avg. Sample Rate: {sum(rates) / len(rates)}")